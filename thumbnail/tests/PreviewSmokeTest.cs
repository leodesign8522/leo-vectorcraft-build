using System;
using System.Drawing;
using System.IO;
using LeoDesign.ThumbnailPreview;
class SmokeTest
{
    static int Main(string[] args)
    {
        if (args.Length != 1) return 2;
        int passed = 0;
        foreach (string label in new[] { "plain", "gzip" })
        {
            using (var s = File.OpenRead(Path.Combine(args[0], label + ".vectorcraft")))
            {
                byte[] png = NativePreview.ExtractPreview(s);
                if (png == null || png.Length < 40 || png[1] != (byte)'P')
                    throw new Exception(label + " preview was not extracted.");
                using (var m = new MemoryStream(png))
                using (var bmp = Image.FromStream(m, false, true))
                {
                    if (bmp.Width != 2 || bmp.Height != 2)
                        throw new Exception(label + " PNG does not decode.");
                }
                Console.WriteLine("PASS " + label + " preview: " + png.Length + " bytes");
                passed++;
            }
        }
        using (var s = File.OpenRead(Path.Combine(args[0], "no-preview.vectorcraft")))
        {
            if (NativePreview.ExtractPreview(s) != null)
                throw new Exception("Unexpected artwork preview in no-preview file.");
            Console.WriteLine("PASS no-preview: safe fallback");
            passed++;
        }
        Console.WriteLine("PASS " + passed + " extract tests");
        return 0;
    }
}
