using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
using LeoDesign.ThumbnailPreview;

class ComSmokeTest
{
    [DllImport("shlwapi.dll", CharSet = CharSet.Unicode, PreserveSig = true)]
    private static extern int SHCreateStreamOnFileEx(
        [MarshalAs(UnmanagedType.LPWStr)] string file, uint mode, uint attrs,
        [MarshalAs(UnmanagedType.Bool)] bool create,
        [MarshalAs(UnmanagedType.Interface)] IStream template,
        [MarshalAs(UnmanagedType.Interface)] out IStream result);

    [DllImport("gdi32.dll")]
    private static extern bool DeleteObject(IntPtr handle);

    static int Main(string[] args)
    {
        if (args.Length != 1) return 2;
        var clsid = new Guid("DA7638F6-36B8-459F-90BC-CEF9A6CFC923");
        var type = Type.GetTypeFromCLSID(clsid, true);
        foreach (string name in new[] { "plain", "gzip" })
        {
            object com = null;
            IStream stream = null;
            try
            {
                com = Activator.CreateInstance(type);
                var initialize = com as IInitializeWithStream;
                var provider = com as IThumbnailProvider;
                if (initialize == null || provider == null)
                    throw new Exception("COM object does not implement the expected shell interfaces.");
                string path = Path.GetFullPath(Path.Combine(args[0], name + ".vectorcraft"));
                int hr = SHCreateStreamOnFileEx(path, 0, 0, false, null, out stream);
                if (hr != 0) Marshal.ThrowExceptionForHR(hr);
                hr = initialize.Initialize(stream, 0);
                if (hr != 0) Marshal.ThrowExceptionForHR(hr);
                IntPtr hbitmap;
                WtsAlphaType alpha;
                hr = provider.GetThumbnail(128, out hbitmap, out alpha);
                if (hr != 0 || hbitmap == IntPtr.Zero)
                    throw new Exception("COM GetThumbnail failed for " + name + " (HRESULT " + hr.ToString("X8") + ")");
                DeleteObject(hbitmap);
                Console.WriteLine("PASS COM thumbnail " + name + ", alpha=" + alpha);
            }
            finally
            {
                if (stream != null) Marshal.ReleaseComObject(stream);
                if (com != null && Marshal.IsComObject(com)) Marshal.ReleaseComObject(com);
            }
        }
        return 0;
    }
}
