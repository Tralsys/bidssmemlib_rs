using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;

/// <summary>
/// C# interop test program for verifying binary layout compatibility with
/// the Rust bids-smemlib implementation.
///
/// Uses the same MemoryMappedFile API that the C# BIDSSMemLib uses internally
/// (MemoryMappedViewAccessor.Read/Write&lt;T&gt;).
///
/// Commands:
///   write-bsmd &lt;mmf-name&gt;   - Write known BSMD data, print "READY", wait for "QUIT"
///   verify-bsmd &lt;mmf-name&gt;  - Read BSMD data and verify against expected values
///   write-panel &lt;mmf-name&gt;  - Write known panel data, print "READY", wait for "QUIT"
///   verify-panel &lt;mmf-name&gt; - Read panel data and verify against expected values
///   dump-layout              - Print struct sizes and field offsets (for debugging)
/// </summary>

// ---- Struct Definitions ----
// These match the C# BIDSSMemLib structs exactly (LayoutKind.Sequential).
// They also must be binary-compatible with the Rust #[repr(C)] structs.

[StructLayout(LayoutKind.Sequential)]
struct Spec
{
    public int B;
    public int P;
    public int A;
    public int J;
    public int C;
}

[StructLayout(LayoutKind.Sequential)]
struct State
{
    public double Z;
    public float V;
    public int T;
    public float BC;
    public float MR;
    public float ER;
    public float BP;
    public float SAP;
    public float I;
}

[StructLayout(LayoutKind.Sequential)]
struct Hand
{
    public int B;
    public int P;
    public int R;
    public int C;
}

[StructLayout(LayoutKind.Sequential)]
struct BIDSSharedMemoryData
{
    public bool IsEnabled;
    public int VersionNum;
    public Spec SpecData;
    public State StateData;
    public Hand HandleData;
    public bool IsDoorClosed;
}

[StructLayout(LayoutKind.Sequential)]
struct PreTrainD
{
    public bool IsEnabled;
    public double Location;
    public double Distance;
    public double Speed;
}

[StructLayout(LayoutKind.Sequential)]
struct OpenD
{
    public bool IsEnabled;
    public int Ver;
    public double Radius;
    public double Cant;
    public double Pitch;
    public double ElapTime;
    public PreTrainD PreTrain;
    public int SelfBCount;
    public int SelfBPosition;
}

[StructLayout(LayoutKind.Sequential)]
struct Hands
{
    public int B;
    public int P;
    public int R;
    public int S;
    public double BPos;
    public double PPos;
}

class Program
{
    // Well-known test data (must match the Rust side exactly)
    static BIDSSharedMemoryData TestBsmd => new BIDSSharedMemoryData
    {
        IsEnabled = true,
        VersionNum = 203,
        SpecData = new Spec { B = 8, P = 5, A = 2, J = 7, C = 10 },
        StateData = new State
        {
            Z = 12345.678,
            V = 80.5f,
            T = 43200000,
            BC = 200.0f,
            MR = 780.0f,
            ER = 490.0f,
            BP = 490.0f,
            SAP = 490.0f,
            I = 150.0f,
        },
        HandleData = new Hand { B = 0, P = 3, R = 1, C = 0 },
        IsDoorClosed = true,
    };

    static OpenD TestOpenD => new OpenD
    {
        IsEnabled = true,
        Ver = 1,
        Radius = 500.0,
        Cant = 105.0,
        Pitch = -15.0,
        ElapTime = 16.67,
        PreTrain = new PreTrainD
        {
            IsEnabled = true,
            Location = 5000.0,
            Distance = 1200.0,
            Speed = 75.0,
        },
        SelfBCount = 5,
        SelfBPosition = 2,
    };

    static int[] TestPanel
    {
        get
        {
            var panel = new int[256];
            for (int i = 0; i < 256; i++) panel[i] = i * 10;
            return panel;
        }
    }

    static int Main(string[] args)
    {
        if (args.Length < 1)
        {
            Console.Error.WriteLine("Usage: InteropTest <command> [mmf-name]");
            return 1;
        }

        try
        {
            return args[0] switch
            {
                "write-bsmd" => WriteBsmd(args[1]),
                "verify-bsmd" => VerifyBsmd(args[1]),
                "write-open-d" => WriteOpenD(args[1]),
                "verify-open-d" => VerifyOpenD(args[1]),
                "write-panel" => WritePanel(args[1]),
                "verify-panel" => VerifyPanel(args[1]),
                "dump-layout" => DumpLayout(),
                _ => Error($"Unknown command: {args[0]}"),
            };
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"ERROR: {ex}");
            return 1;
        }
    }

    static int WriteBsmd(string name)
    {
        var bsmd = TestBsmd;
        long capacity = 4096;
        using var mmf = MemoryMappedFile.CreateOrOpen(name, capacity);
        using var accessor = mmf.CreateViewAccessor(0, capacity);
        accessor.Write(0, ref bsmd);
        Console.WriteLine("READY");
        Console.Out.Flush();
        // Wait for QUIT signal
        while (true)
        {
            var line = Console.ReadLine();
            if (line == null || line.Trim() == "QUIT") break;
        }
        return 0;
    }

    static int VerifyBsmd(string name)
    {
        long capacity = 4096;
        using var mmf = MemoryMappedFile.CreateOrOpen(name, capacity);
        using var accessor = mmf.CreateViewAccessor(0, capacity);
        accessor.Read(0, out BIDSSharedMemoryData bsmd);

        var expected = TestBsmd;
        int errors = 0;
        errors += Check("IsEnabled", bsmd.IsEnabled, expected.IsEnabled);
        errors += Check("VersionNum", bsmd.VersionNum, expected.VersionNum);
        errors += Check("Spec.B", bsmd.SpecData.B, expected.SpecData.B);
        errors += Check("Spec.P", bsmd.SpecData.P, expected.SpecData.P);
        errors += Check("Spec.A", bsmd.SpecData.A, expected.SpecData.A);
        errors += Check("Spec.J", bsmd.SpecData.J, expected.SpecData.J);
        errors += Check("Spec.C", bsmd.SpecData.C, expected.SpecData.C);
        errors += Check("State.Z", bsmd.StateData.Z, expected.StateData.Z);
        errors += Check("State.V", bsmd.StateData.V, expected.StateData.V);
        errors += Check("State.T", bsmd.StateData.T, expected.StateData.T);
        errors += Check("State.BC", bsmd.StateData.BC, expected.StateData.BC);
        errors += Check("State.MR", bsmd.StateData.MR, expected.StateData.MR);
        errors += Check("State.ER", bsmd.StateData.ER, expected.StateData.ER);
        errors += Check("State.BP", bsmd.StateData.BP, expected.StateData.BP);
        errors += Check("State.SAP", bsmd.StateData.SAP, expected.StateData.SAP);
        errors += Check("State.I", bsmd.StateData.I, expected.StateData.I);
        errors += Check("Hand.B", bsmd.HandleData.B, expected.HandleData.B);
        errors += Check("Hand.P", bsmd.HandleData.P, expected.HandleData.P);
        errors += Check("Hand.R", bsmd.HandleData.R, expected.HandleData.R);
        errors += Check("Hand.C", bsmd.HandleData.C, expected.HandleData.C);
        errors += Check("IsDoorClosed", bsmd.IsDoorClosed, expected.IsDoorClosed);

        if (errors == 0)
        {
            Console.WriteLine("OK: All BSMD fields match");
            return 0;
        }
        Console.Error.WriteLine($"FAIL: {errors} field(s) mismatched");
        return 1;
    }

    static int WriteOpenD(string name)
    {
        var openD = TestOpenD;
        long capacity = 4096;
        using var mmf = MemoryMappedFile.CreateOrOpen(name, capacity);
        using var accessor = mmf.CreateViewAccessor(0, capacity);
        accessor.Write(0, ref openD);
        Console.WriteLine("READY");
        Console.Out.Flush();
        while (true)
        {
            var line = Console.ReadLine();
            if (line == null || line.Trim() == "QUIT") break;
        }
        return 0;
    }

    static int VerifyOpenD(string name)
    {
        long capacity = 4096;
        using var mmf = MemoryMappedFile.CreateOrOpen(name, capacity);
        using var accessor = mmf.CreateViewAccessor(0, capacity);
        accessor.Read(0, out OpenD openD);

        var expected = TestOpenD;
        int errors = 0;
        errors += Check("IsEnabled", openD.IsEnabled, expected.IsEnabled);
        errors += Check("Ver", openD.Ver, expected.Ver);
        errors += Check("Radius", openD.Radius, expected.Radius);
        errors += Check("Cant", openD.Cant, expected.Cant);
        errors += Check("Pitch", openD.Pitch, expected.Pitch);
        errors += Check("ElapTime", openD.ElapTime, expected.ElapTime);
        errors += Check("PreTrain.IsEnabled", openD.PreTrain.IsEnabled, expected.PreTrain.IsEnabled);
        errors += Check("PreTrain.Location", openD.PreTrain.Location, expected.PreTrain.Location);
        errors += Check("PreTrain.Distance", openD.PreTrain.Distance, expected.PreTrain.Distance);
        errors += Check("PreTrain.Speed", openD.PreTrain.Speed, expected.PreTrain.Speed);
        errors += Check("SelfBCount", openD.SelfBCount, expected.SelfBCount);
        errors += Check("SelfBPosition", openD.SelfBPosition, expected.SelfBPosition);

        if (errors == 0)
        {
            Console.WriteLine("OK: All OpenD fields match");
            return 0;
        }
        Console.Error.WriteLine($"FAIL: {errors} field(s) mismatched");
        return 1;
    }

    static int WritePanel(string name)
    {
        long capacity = 4096;
        using var mmf = MemoryMappedFile.CreateOrOpen(name, capacity);
        using var accessor = mmf.CreateViewAccessor(0, capacity);

        var panel = TestPanel;
        // Write length prefix (i32) followed by elements (matching ArrayDataSMemCtrler layout)
        accessor.Write(0, panel.Length);
        for (int i = 0; i < panel.Length; i++)
        {
            accessor.Write(4 + i * 4, panel[i]);
        }

        Console.WriteLine("READY");
        Console.Out.Flush();
        while (true)
        {
            var line = Console.ReadLine();
            if (line == null || line.Trim() == "QUIT") break;
        }
        return 0;
    }

    static int VerifyPanel(string name)
    {
        long capacity = 4096;
        using var mmf = MemoryMappedFile.CreateOrOpen(name, capacity);
        using var accessor = mmf.CreateViewAccessor(0, capacity);

        var expected = TestPanel;
        int length = accessor.ReadInt32(0);
        int errors = 0;
        errors += Check("Panel.Length", length, expected.Length);

        for (int i = 0; i < Math.Min(length, expected.Length); i++)
        {
            int val = accessor.ReadInt32(4 + i * 4);
            errors += Check($"Panel[{i}]", val, expected[i]);
        }

        if (errors == 0)
        {
            Console.WriteLine("OK: All Panel data match");
            return 0;
        }
        Console.Error.WriteLine($"FAIL: {errors} value(s) mismatched");
        return 1;
    }

    static int DumpLayout()
    {
        Console.WriteLine($"sizeof(Spec) = {Marshal.SizeOf<Spec>()}");
        Console.WriteLine($"sizeof(State) = {Marshal.SizeOf<State>()}");
        Console.WriteLine($"sizeof(Hand) = {Marshal.SizeOf<Hand>()}");
        Console.WriteLine($"sizeof(BIDSSharedMemoryData) = {Marshal.SizeOf<BIDSSharedMemoryData>()}");
        Console.WriteLine($"sizeof(PreTrainD) = {Marshal.SizeOf<PreTrainD>()}");
        Console.WriteLine($"sizeof(OpenD) = {Marshal.SizeOf<OpenD>()}");
        Console.WriteLine($"sizeof(Hands) = {Marshal.SizeOf<Hands>()}");

        Console.WriteLine();
        Console.WriteLine($"offsetof(BSMD.IsEnabled) = {Marshal.OffsetOf<BIDSSharedMemoryData>(nameof(BIDSSharedMemoryData.IsEnabled))}");
        Console.WriteLine($"offsetof(BSMD.VersionNum) = {Marshal.OffsetOf<BIDSSharedMemoryData>(nameof(BIDSSharedMemoryData.VersionNum))}");
        Console.WriteLine($"offsetof(BSMD.SpecData) = {Marshal.OffsetOf<BIDSSharedMemoryData>(nameof(BIDSSharedMemoryData.SpecData))}");
        Console.WriteLine($"offsetof(BSMD.StateData) = {Marshal.OffsetOf<BIDSSharedMemoryData>(nameof(BIDSSharedMemoryData.StateData))}");
        Console.WriteLine($"offsetof(BSMD.HandleData) = {Marshal.OffsetOf<BIDSSharedMemoryData>(nameof(BIDSSharedMemoryData.HandleData))}");
        Console.WriteLine($"offsetof(BSMD.IsDoorClosed) = {Marshal.OffsetOf<BIDSSharedMemoryData>(nameof(BIDSSharedMemoryData.IsDoorClosed))}");

        return 0;
    }

    static int Check<T>(string name, T actual, T expected) where T : notnull
    {
        if (actual.Equals(expected)) return 0;
        Console.Error.WriteLine($"  MISMATCH {name}: expected={expected}, actual={actual}");
        return 1;
    }

    static int Error(string msg)
    {
        Console.Error.WriteLine(msg);
        return 1;
    }
}
