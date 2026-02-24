from __future__ import annotations

from argparse import ArgumentParser, Namespace
from pathlib import Path
from typing import List, Optional, Sequence

from . import _native

__version__ = "4.6.1.post1"


class VerificationException(RuntimeError):
    pass


class VerificationFailed:
    def __init__(self, exception: Exception, in_file: Path):
        self.exception = exception
        self.in_file = in_file


def _as_str_path(path: Optional[Path]) -> Optional[str]:
    if path is None:
        return None
    return str(path)


def _arg(args: Namespace, *names: str, default):
    for name in names:
        if hasattr(args, name):
            return getattr(args, name)
    return default


def _expand_files(items: Sequence[str]) -> List[Path]:
    expanded: List[Path] = []
    for item in items:
        path = Path(item)
        if path.is_dir():
            expanded.extend(candidate for candidate in sorted(path.rglob("*")) if candidate.is_file())
        elif path.exists():
            expanded.append(path)
    return expanded


def compress(filePath, outputDir, args, work=None, amountOfTastkQueued=None):
    compression_level = 18 if _arg(args, "level", default=None) is None else int(_arg(args, "level", default=18))
    processed = _native.compress(
        [str(Path(filePath))],
        output_dir=_as_str_path(Path(outputDir) if outputDir is not None else None),
        level=compression_level,
        long_distance_mode=bool(_arg(args, "long", default=False)),
        block=bool(_arg(args, "block", default=False)),
        solid=bool(_arg(args, "solid", default=False)),
        block_size_exponent=int(_arg(args, "bs", default=20)),
        verify=bool(_arg(args, "verify", default=False)),
        quick_verify=bool(_arg(args, "quick_verify", default=False)),
        keep=bool(_arg(args, "keep", default=False)),
        fix_padding=bool(_arg(args, "fix_padding", default=False)),
        parse_cnmt=bool(_arg(args, "parseCnmt", "parse_cnmt", default=False)),
        always_parse_cnmt=bool(_arg(args, "alwaysParseCnmt", "always_parse_cnmt", default=False)),
        multi=int(_arg(args, "multi", default=4)),
        threads=int(_arg(args, "threads", default=-1)),
        overwrite=bool(_arg(args, "overwrite", default=False)),
        rm_old_version=bool(_arg(args, "rm_old_version", default=False)),
        rm_source=bool(_arg(args, "rm_source", default=False)),
    )
    return Path(processed[0]) if processed else None


def decompress(filePath, outputDir, fixPadding, statusReportInfo=None):
    _native.decompress(
        [str(Path(filePath))],
        output_dir=_as_str_path(Path(outputDir) if outputDir is not None else None),
        fix_padding=bool(fixPadding),
    )


def verify(
    filePath,
    fixPadding,
    raiseVerificationException,
    raisePfs0Exception,
    originalFilePath=None,
    statusReportInfo=None,
    pleaseNoPrint=None,
):
    try:
        _native.verify([str(Path(filePath))], fix_padding=bool(fixPadding))
    except RuntimeError as exc:
        if raiseVerificationException:
            raise VerificationException(str(exc)) from exc
        return


def extract(file_paths: Sequence[str], output_dir: Optional[str] = None, extract_regex: Optional[str] = None):
    return _native.extract(
        [str(Path(path)) for path in file_paths],
        output_dir=output_dir,
        extract_regex=extract_regex,
    )


def create(output_file: str, sources: Sequence[str], fix_padding: bool = False):
    return _native.create(output_file=output_file, sources=[str(Path(path)) for path in sources], fix_padding=fix_padding)


def titlekeys(file_paths: Sequence[str]):
    return _native.titlekeys([str(Path(path)) for path in file_paths])


def undupe_files(
    file_paths: Sequence[str],
    output_dir: Optional[str] = None,
    dry_run: bool = False,
    rename: bool = False,
    hardlink: bool = False,
    priority_list: Optional[str] = None,
    whitelist: Optional[str] = None,
    blacklist: Optional[str] = None,
    old_versions: bool = False,
):
    return _native.undupe(
        [str(Path(path)) for path in file_paths],
        output_dir=output_dir,
        dry_run=dry_run,
        rename=rename,
        hardlink=hardlink,
        priority_list=priority_list,
        whitelist=whitelist,
        blacklist=blacklist,
        old_versions=old_versions,
    )


def _build_parser() -> ArgumentParser:
    parser = ArgumentParser()
    parser.add_argument("file", nargs="*")
    parser.add_argument("-C", action="store_true")
    parser.add_argument("-D", action="store_true")
    parser.add_argument("-l", "--level", type=int, default=18)
    parser.add_argument("-L", "--long", action="store_true", default=False)
    parser.add_argument("-B", "--block", action="store_true", default=False)
    parser.add_argument("-S", "--solid", action="store_true", default=False)
    parser.add_argument("-s", "--bs", type=int, default=20)
    parser.add_argument("-V", "--verify", action="store_true", default=False)
    parser.add_argument("-Q", "--quick-verify", action="store_true", default=False)
    parser.add_argument("-K", "--keep", action="store_true", default=False)
    parser.add_argument("-F", "--fix-padding", action="store_true", default=False)
    parser.add_argument("-p", "--parseCnmt", action="store_true", default=False)
    parser.add_argument("-P", "--alwaysParseCnmt", action="store_true", default=False)
    parser.add_argument("-t", "--threads", type=int, default=-1)
    parser.add_argument("-m", "--multi", type=int, default=4)
    parser.add_argument("-o", "--output")
    parser.add_argument("-w", "--overwrite", action="store_true", default=False)
    parser.add_argument("-r", "--rm-old-version", action="store_true", default=False)
    parser.add_argument("--rm-source", action="store_true", default=False)
    parser.add_argument("-x", "--extract", action="store_true")
    parser.add_argument("--extractregex", type=str, default="")
    parser.add_argument("--titlekeys", action="store_true", default=False)
    parser.add_argument("--undupe", action="store_true")
    parser.add_argument("--undupe-dryrun", action="store_true")
    parser.add_argument("--undupe-rename", action="store_true")
    parser.add_argument("--undupe-hardlink", action="store_true")
    parser.add_argument("--undupe-prioritylist", type=str, default="")
    parser.add_argument("--undupe-whitelist", type=str, default="")
    parser.add_argument("--undupe-blacklist", type=str, default="")
    parser.add_argument("--undupe-old-versions", action="store_true", default=False)
    parser.add_argument("-c", "--create")
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)
    files = _expand_files(args.file)
    output_dir = Path(args.output).resolve() if args.output else None
    if output_dir is not None and not output_dir.exists():
        raise RuntimeError(f'Output directory "{output_dir}" does not exist')

    if args.C:
        for file_path in files:
            if file_path.suffix.lower() not in {".nsp", ".xci", ".nca"}:
                continue
            compress(file_path, output_dir if output_dir is not None else file_path.parent, args)
        return 0

    if args.D:
        for file_path in files:
            if file_path.suffix.lower() not in {".nsz", ".xcz", ".ncz"}:
                continue
            decompress(file_path, output_dir if output_dir is not None else file_path.parent, args.fix_padding)
        return 0

    if args.extract:
        extract([str(path) for path in files], _as_str_path(output_dir), args.extractregex or None)
        return 0

    if args.create:
        create(args.create, [str(path) for path in files], fix_padding=args.fix_padding)
        return 0

    if args.titlekeys:
        titlekeys([str(path) for path in files])
        return 0

    if args.undupe or args.undupe_dryrun:
        undupe_files(
            [str(path) for path in files],
            output_dir=_as_str_path(output_dir),
            dry_run=args.undupe_dryrun,
            rename=args.undupe_rename,
            hardlink=args.undupe_hardlink,
            priority_list=args.undupe_prioritylist or None,
            whitelist=args.undupe_whitelist or None,
            blacklist=args.undupe_blacklist or None,
            old_versions=args.undupe_old_versions,
        )
        return 0

    if args.verify:
        for file_path in files:
            verify(file_path, args.fix_padding, True, True)
        return 0

    parser.print_help()
    return 0


__all__ = [
    "__version__",
    "VerificationException",
    "VerificationFailed",
    "compress",
    "decompress",
    "verify",
    "extract",
    "create",
    "titlekeys",
    "undupe_files",
    "main",
]
