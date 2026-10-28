using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
using System.IO.Compression;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
using System.Text;
using System.Text.RegularExpressions;

namespace LeoDesign.ThumbnailPreview
{
    // Windows Shell's IInitializeWithStream and IThumbnailProvider.
    [ComVisible(true)]
    [Guid("B824B49D-22AC-4161-AC8A-9916E8FA3F7F")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IInitializeWithStream
    {
        [PreserveSig]
        int Initialize([MarshalAs(UnmanagedType.Interface)] IStream stream, uint mode);
    }

    [ComVisible(true)]
    [Guid("E357FCCD-A995-4576-B01F-234630154E96")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IThumbnailProvider
    {
        [PreserveSig]
        int GetThumbnail(uint cx, out IntPtr phbmp, out WtsAlphaType alpha);
    }

    public enum WtsAlphaType { Unknown = 0, Rgb = 1, Argb = 2 }

    [ComVisible(true)]
    [Guid("DA7638F6-36B8-459F-90BC-CEF9A6CFC923")]
    [ProgId("LeoDesign.VectorCraftThumbnailPreview")]
    [ClassInterface(ClassInterfaceType.None)]
    public sealed class VectorCraftThumbnailProvider : IInitializeWithStream, IThumbnailProvider
    {
        private IStream input;
        private const int E_FAIL = unchecked((int)0x80004005);

        public int Initialize(IStream stream, uint mode)
        {
            if (stream == null) return unchecked((int)0x80070057);
            input = stream;
            return 0;
        }

        public int GetThumbnail(uint cx, out IntPtr phbmp, out WtsAlphaType alpha)
        {
            phbmp = IntPtr.Zero;
            alpha = WtsAlphaType.Rgb;
            if (input == null) return E_FAIL;
            try
            {
                using (var com = new ComInputStream(input))
                {
                    byte[] png = NativePreview.ExtractPreview(com);
                    if (png == null) return E_FAIL;
                    using (var ms = new MemoryStream(png, false))
                    using (var src = Image.FromStream(ms, false, true))
                    {
                        int side = (int)Math.Min(Math.Max(cx, 32u), 1024u);
                        using (var picture = new Bitmap(side, side, PixelFormat.Format24bppRgb))
                        using (var g = Graphics.FromImage(picture))
                        {
                            g.Clear(Color.White);
                            g.CompositingQuality = CompositingQuality.HighQuality;
                            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
                            g.SmoothingMode = SmoothingMode.HighQuality;
                            double factor = Math.Min((double)side / src.Width, (double)side / src.Height);
                            int w = Math.Max(1, (int)Math.Round(src.Width * factor));
                            int h = Math.Max(1, (int)Math.Round(src.Height * factor));
                            g.DrawImage(src, new Rectangle((side - w) / 2, (side - h) / 2, w, h));
                            phbmp = picture.GetHbitmap(Color.White);
                            return 0; // Windows owns and DeleteObject-s the returned HBITMAP.
                        }
                    }
                }
            }
            catch (Exception) { return E_FAIL; }
        }
    }

    // Lightweight read-only adapter: never writes to, modifies or locks the original document.
    internal sealed class ComInputStream : Stream
    {
        private readonly IStream source;
        private readonly IntPtr readCount = Marshal.AllocHGlobal(4);
        public ComInputStream(IStream source) { this.source = source; }
        public override int Read(byte[] b, int offset, int count)
        {
            if (count == 0) return 0;
            byte[] temp = new byte[count];
            Marshal.WriteInt32(readCount, 0);
            source.Read(temp, count, readCount);
            int len = Marshal.ReadInt32(readCount);
            if (len < 0 || len > count) throw new IOException("Invalid IStream read.");
            if (len > 0) Buffer.BlockCopy(temp, 0, b, offset, len);
            return len;
        }
        public override bool CanRead { get { return true; } }
        public override bool CanSeek { get { return true; } }
        public override bool CanWrite { get { return false; } }
        public override long Length { get { throw new NotSupportedException(); } }
        public override long Position { get { throw new NotSupportedException(); } set { Seek(value, SeekOrigin.Begin); } }
        public override long Seek(long offset, SeekOrigin origin)
        {
            IntPtr p = Marshal.AllocHGlobal(8);
            try { source.Seek(offset, (int)origin, p); return Marshal.ReadInt64(p); }
            finally { Marshal.FreeHGlobal(p); }
        }
        public override void SetLength(long value) { throw new NotSupportedException(); }
        public override void Flush() { }
        public override void Write(byte[] b, int offset, int count) { throw new NotSupportedException(); }
        protected override void Dispose(bool disposing)
        {
            Marshal.FreeHGlobal(readCount);
            base.Dispose(disposing);
        }
    }

    public static class NativePreview
    {
        // Native .vectorcraft: UTF-8 JSON or gzip, with an OPTIONAL "preview" PNG
        // before "document". Never attempt to render untrusted documents inside Explorer.
        private const int MaxHeadBytes = 16 * 1024 * 1024;
        private const int MaxPngBytes = 8 * 1024 * 1024;
        private static readonly byte[] Signature = { 137, 80, 78, 71, 13, 10, 26, 10 };

        public static byte[] ExtractPreview(Stream file)
        {
            if (file == null || !file.CanRead) return null;
            byte[] start = new byte[2];
            int n = file.Read(start, 0, 2);
            if (n != 2) return null;
            file.Seek(0, SeekOrigin.Begin);
            bool compressed = start[0] == 0x1f && start[1] == 0x8b;
            using (Stream content = compressed
                ? (Stream)new GZipStream(file, CompressionMode.Decompress, true)
                : new NonClosingStream(file))
            using (var head = new MemoryStream())
            {
                var chunk = new byte[16 * 1024];
                while (head.Length < MaxHeadBytes)
                {
                    int count = content.Read(chunk, 0, Math.Min(chunk.Length, MaxHeadBytes - (int)head.Length));
                    if (count <= 0) break;
                    head.Write(chunk, 0, count);
                    string text = Encoding.UTF8.GetString(head.GetBuffer(), 0, (int)head.Length);
                    int doc = text.IndexOf("\"document\"", StringComparison.Ordinal);
                    int preview = text.IndexOf("\"preview\"", StringComparison.Ordinal);
                    if (doc >= 0 && (preview < 0 || preview > doc)) return null;
                    if (preview >= 0)
                    {
                        int brace = text.IndexOf('{', preview);
                        int close = brace < 0 ? -1 : text.IndexOf('}', brace);
                        if (close < 0) continue;
                        string part = text.Substring(brace, close - brace + 1);
                        if (!Regex.IsMatch(part, "\"mime\"\\s*:\\s*\"image/png\"", RegexOptions.CultureInvariant)) return null;
                        Match m = Regex.Match(part, "\"data\"\\s*:\\s*\"([A-Za-z0-9+/=]+)\"", RegexOptions.CultureInvariant);
                        if (!m.Success || m.Groups[1].Length > (MaxPngBytes * 4 / 3 + 8)) return null;
                        byte[] png = Convert.FromBase64String(m.Groups[1].Value);
                        if (png.Length < 24 || png.Length > MaxPngBytes) return null;
                        for (int i = 0; i < Signature.Length; i++)
                            if (png[i] != Signature[i]) return null;
                        return png;
                    }
                }
                return null;
            }
        }
    }

    internal sealed class NonClosingStream : Stream
    {
        private readonly Stream inner;
        public NonClosingStream(Stream inner) { this.inner = inner; }
        public override bool CanRead { get { return inner.CanRead; } }
        public override bool CanSeek { get { return inner.CanSeek; } }
        public override bool CanWrite { get { return false; } }
        public override long Length { get { return inner.Length; } }
        public override long Position { get { return inner.Position; } set { inner.Position = value; } }
        public override int Read(byte[] b, int o, int n) { return inner.Read(b, o, n); }
        public override long Seek(long d, SeekOrigin o) { return inner.Seek(d, o); }
        public override void Flush() { }
        public override void SetLength(long v) { throw new NotSupportedException(); }
        public override void Write(byte[] b, int o, int n) { throw new NotSupportedException(); }
    }
}
