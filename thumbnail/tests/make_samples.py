import base64, gzip, json, pathlib, struct, sys, zlib

dst = pathlib.Path(sys.argv[1])
dst.mkdir(parents=True, exist_ok=True)

def chunk(name, payload):
    return struct.pack(">I", len(payload)) + name + payload + struct.pack(">I", zlib.crc32(name + payload) & 0xffffffff)

# Valid 2x2 RGBA PNG, generated without third-party dependencies.
pixels = (
    bytes([0, 255, 0, 0, 255, 0, 255, 0, 255]) +    # filter 0, red, green
    bytes([0, 0, 0, 255, 255, 255, 255, 0, 255])     # filter 0, blue, yellow
)
png = (b"\x89PNG\r\n\x1a\n" +
       chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 2, 8, 6, 0, 0, 0)) +
       chunk(b"IDAT", zlib.compress(pixels)) +
       chunk(b"IEND", b""))
native = {"format":"vectorcraft","version":4,"generator":"thumbnail-test",
          "preview":{"mime":"image/png","data":base64.b64encode(png).decode("ascii")},
          "document":{"title":"Smoke test"},"images":{}}
encoded = json.dumps(native, separators=(",", ":")).encode("utf8")
(dst / "plain.vectorcraft").write_bytes(encoded)
(dst / "gzip.vectorcraft").write_bytes(gzip.compress(encoded))
del native["preview"]
(dst / "no-preview.vectorcraft").write_bytes(json.dumps(native).encode("utf8"))
print("Prepared valid PNG preview fixture and gzip/plain test documents.")
