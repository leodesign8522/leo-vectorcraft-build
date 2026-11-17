"""Register Leo Design's native circle and arc tools."""
from pathlib import Path
from shutil import copyfile
import sys

source = Path(__file__).resolve().parent
tools = Path(sys.argv[1]) / "crates/tools/src"


def replace(file, before, after):
    path = tools / file
    text = path.read_text(encoding="utf-8")
    if text.count(before) != 1:
        raise SystemExit(f"Unexpected geometry tool anchor in {file}: {before}")
    path.write_text(text.replace(before, after, 1), encoding="utf-8", newline="\n")


for file in ("leo_circle_tool.rs", "leo_circle_geometry.rs"):
    copyfile(source / file, tools / file)

replace("lib.rs", "mod input;", "mod input;\nmod leo_circle_tool;")
replace("lib.rs",
        '        "dimension" => Box::new(dimension::DimensionTool::default()),',
        '''        "dimension" => Box::new(dimension::DimensionTool::default()),
        id if leo_circle_tool::IDS.contains(&id) => Box::new(leo_circle_tool::CircleTool::new(id)),''')
replace("catalog.rs",
        '''    &[t("freeTransform", "Free Transform Tool", Some("E"), "tool-free-transform"),''',
        '''    &[
        t("leoCirclePoints", "Leo Design 按点画圆", None, "tool-ellipse"),
        t("leoTangentCircle", "Leo Design 相切圆", None, "tool-ellipse"),
        t("leoCurvatureCircle", "Leo Design 曲率圆", None, "tool-ellipse"),
        t("leoArcPoints", "Leo Design 按点圆弧", None, "tool-arc"),
    ],
    &[t("freeTransform", "Free Transform Tool", Some("E"), "tool-free-transform"),''')
