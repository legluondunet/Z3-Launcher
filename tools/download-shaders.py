"""Install the official GLSL shader archive into the launcher's data directory."""
import pathlib
import shutil
import sys
import tempfile
import urllib.request
import zipfile

URL = "https://github.com/libretro/glsl-shaders/archive/refs/heads/master.zip"
SOURCE = "https://github.com/libretro/glsl-shaders"


def install(destination):
    destination = pathlib.Path(destination)
    marker = destination / ".launcher-shaders-source"
    if marker.is_file() and marker.read_text().strip() == SOURCE and all(
        (destination / name).is_file() for name in ("stock.glsl", "nearest.glslp", "bilinear.glslp")
    ):
        print("GLSL shaders already installed:", destination, flush=True)
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="shader-download-", dir=destination.parent) as temporary:
        temporary = pathlib.Path(temporary)
        archive = temporary / "shaders.zip"
        print("Downloading", URL, flush=True)
        with urllib.request.urlopen(URL, timeout=120) as response, archive.open("wb") as out:
            shutil.copyfileobj(response, out)
        staged = temporary / "shaders"
        staged.mkdir()
        # Preserve additional files if recovering an incomplete installation.
        if destination.exists():
            shutil.copytree(destination, staged, dirs_exist_ok=True)
        with zipfile.ZipFile(archive) as package:
            roots = {pathlib.PurePosixPath(item.filename).parts[0] for item in package.infolist() if item.filename}
            if len(roots) != 1:
                raise ValueError("Unexpected shader archive layout")
            for item in package.infolist():
                parts = pathlib.PurePosixPath(item.filename).parts
                if not parts or len(parts) == 1:
                    continue
                if item.filename.startswith("/") or any(part in ("..", "") or "\\" in part or ":" in part for part in parts):
                    raise ValueError("Invalid shader archive path")
                if (item.external_attr >> 16) & 0o170000 == 0o120000:
                    raise ValueError("Symlinks are not supported in the shader archive")
                target = staged.joinpath(*parts[1:])
                if item.is_dir():
                    target.mkdir(parents=True, exist_ok=True)
                else:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    with package.open(item) as src, target.open("wb") as out:
                        shutil.copyfileobj(src, out)
        for name in ("stock.glsl", "nearest.glslp", "bilinear.glslp"):
            if not (staged / name).is_file():
                raise ValueError("Shader archive is missing " + name)
        (staged / ".launcher-shaders-source").write_text(SOURCE + "\n")
        backup = temporary / "previous"
        if destination.exists():
            destination.rename(backup)
        try:
            staged.rename(destination)
        except BaseException:
            if backup.exists():
                backup.rename(destination)
            raise
    print("GLSL shaders installed:", destination, flush=True)


if __name__ == "__main__":
    install(sys.argv[1])
