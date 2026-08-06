// ChmExtract.cs - extract a .chm via the Windows InfoTech Storage System (itss.dll) COM provider.
// Windows performs the LZX decompression; we just walk the IStorage tree and dump streams.
using System;
using System.IO;
using System.Text;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
using STATSTG = System.Runtime.InteropServices.ComTypes.STATSTG;

static class ChmExtract
{
    static readonly Guid CLSID_ITStorage = new Guid("5d02926a-212e-11d0-9df9-00a0c922e6ec");

    [ComImport, Guid("88cc31de-27ab-11d0-9df9-00a0c922e6ec"),
     InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IITStorage
    {
        [PreserveSig] int StgCreateDocfile([MarshalAs(UnmanagedType.LPWStr)] string n, uint m, uint r, out IStorage s);
        [PreserveSig] int StgCreateDocfileOnILockBytes(IntPtr lb, uint m, uint r, out IStorage s);
        [PreserveSig] int StgIsStorageFile([MarshalAs(UnmanagedType.LPWStr)] string n);
        [PreserveSig] int StgIsStorageILockBytes(IntPtr lb);
        [PreserveSig] int StgOpenStorage([MarshalAs(UnmanagedType.LPWStr)] string n, IStorage pri,
                                         uint m, IntPtr snb, uint r, out IStorage s);
    }

    [ComImport, Guid("0000000b-0000-0000-C000-000000000046"),
     InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IStorage
    {
        void _CreateStream();
        [PreserveSig] int OpenStream([MarshalAs(UnmanagedType.LPWStr)] string name, IntPtr r1,
                                     uint mode, uint r2, out IStream stm);
        void _CreateStorage();
        [PreserveSig] int OpenStorage([MarshalAs(UnmanagedType.LPWStr)] string name, IStorage pri,
                                      uint mode, IntPtr snb, uint r, out IStorage stg);
        void _CopyTo();
        void _MoveElementTo();
        void _Commit();
        void _Revert();
        [PreserveSig] int EnumElements(uint r1, IntPtr r2, uint r3, out IEnumSTATSTG e);
    }

    [ComImport, Guid("0000000d-0000-0000-C000-000000000046"),
     InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IEnumSTATSTG
    {
        [PreserveSig] int Next(uint celt,
            [Out, MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 0)] STATSTG[] rg,
            out uint fetched);
    }

    const uint STGM_READ_SHARE_DENY_WRITE = 0x00000020;   // STGM_READ | STGM_SHARE_DENY_WRITE
    const int STGTY_STORAGE = 1, STGTY_STREAM = 2;

    static int nFiles = 0; static long nBytes = 0; static int nErr = 0;
    static string root;

    static int Main(string[] a)
    {
        if (a.Length < 2) { Console.Error.WriteLine("usage: ChmExtract <chm> <outdir>"); return 2; }
        string chm = Path.GetFullPath(a[0]);
        root = Path.GetFullPath(a[1]);
        Directory.CreateDirectory(root);

        var t = Type.GetTypeFromCLSID(CLSID_ITStorage, true);
        var its = (IITStorage)Activator.CreateInstance(t);

        IStorage stg;
        int hr = its.StgOpenStorage(chm, null, STGM_READ_SHARE_DENY_WRITE, IntPtr.Zero, 0, out stg);
        if (hr != 0) { Console.Error.WriteLine("StgOpenStorage failed 0x" + hr.ToString("X8")); return 3; }

        Walk(stg, "");
        Console.WriteLine("files=" + nFiles + " bytes=" + nBytes + " errors=" + nErr);
        return 0;
    }

    static void Walk(IStorage stg, string prefix)
    {
        IEnumSTATSTG en;
        if (stg.EnumElements(0, IntPtr.Zero, 0, out en) != 0) return;

        var one = new STATSTG[1];
        uint got;
        while (en.Next(1, one, out got) == 0 && got == 1)
        {
            string name = one[0].pwcsName;
            int type = one[0].type;
            if (string.IsNullOrEmpty(name)) continue;
            string rel = prefix.Length == 0 ? name : prefix + "/" + name;

            if (type == STGTY_STORAGE)
            {
                IStorage sub;
                if (stg.OpenStorage(name, null, STGM_READ_SHARE_DENY_WRITE, IntPtr.Zero, 0, out sub) == 0)
                { Walk(sub, rel); Marshal.ReleaseComObject(sub); }
                else nErr++;
            }
            else if (type == STGTY_STREAM)
            {
                IStream stm;
                if (stg.OpenStream(name, IntPtr.Zero, STGM_READ_SHARE_DENY_WRITE, 0, out stm) == 0)
                { Dump(stm, rel); Marshal.ReleaseComObject(stm); }
                else nErr++;
            }
        }
        Marshal.ReleaseComObject(en);
    }

    static void Dump(IStream stm, string rel)
    {
        string safe = Sanitize(rel);
        string path = Path.Combine(root, safe);
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path));
            IntPtr pRead = Marshal.AllocHGlobal(4);
            try
            {
                using (var fs = new FileStream(path, FileMode.Create, FileAccess.Write))
                {
                    var buf = new byte[65536];
                    while (true)
                    {
                        stm.Read(buf, buf.Length, pRead);
                        int n = Marshal.ReadInt32(pRead);
                        if (n <= 0) break;
                        fs.Write(buf, 0, n);
                        nBytes += n;
                    }
                }
            }
            finally { Marshal.FreeHGlobal(pRead); }
            nFiles++;
        }
        catch (Exception ex) { nErr++; Console.Error.WriteLine("ERR " + rel + " :: " + ex.Message); }
    }

    // CHM internal names carry chars that are legal in ITSS but not on NTFS.
    static string Sanitize(string rel)
    {
        var sb = new StringBuilder();
        foreach (char c in rel)
        {
            if (c == '/') { sb.Append(Path.DirectorySeparatorChar); continue; }
            if (c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|' || c < 32)
            { sb.Append('_'); continue; }
            sb.Append(c);
        }
        var s = sb.ToString().TrimStart(Path.DirectorySeparatorChar);
        return s.Length == 0 ? "_root_" : s;
    }
}
