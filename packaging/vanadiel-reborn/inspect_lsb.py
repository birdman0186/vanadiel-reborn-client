import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path


SETTINGS = {
    'network': ('LOGIN_AUTH_PORT', 'LOGIN_DATA_PORT', 'LOGIN_VIEW_PORT', 'MAP_PORT'),
    'login': ('CLIENT_VER', 'VER_LOCK'),
}
VERSION = re.compile(r"[0-9]{8}_[0-9]+")
INTEGER = re.compile(r"[0-9]+")


def safe_candidates(text, keys):
    found = {}
    for line in text.splitlines():
        for key in keys:
            match = re.match(r'\s*' + re.escape(key) + r'\s*=\s*(.*?)\s*,?\s*(?:--.*)?$', line)
            if not match:
                continue
            raw = match.group(1).rstrip(',').strip()
            if INTEGER.fullmatch(raw):
                value = int(raw)
            elif len(raw) > 1 and raw[0] == raw[-1] and raw[0] in "\"'" and VERSION.fullmatch(raw[1:-1]):
                value = raw[1:-1]
            else:
                value = None
            found.setdefault(key, []).append(value)
    return found


def source_revision(root):
    result = subprocess.run(
        ['git', '-C', str(root), 'rev-parse', 'HEAD'],
        capture_output=True, text=True, check=False,
    )
    sha = result.stdout.strip()
    return sha if result.returncode == 0 and re.fullmatch(r'[0-9a-f]{40,64}', sha) else None


def inspect(server_root, dat_root=None):
    server_root = Path(server_root).resolve(strict=True)
    if not (server_root / 'settings' / 'default').is_dir():
        raise ValueError('The server path must contain settings/default.')
    report = {
        'schema_version': 1,
        'server_source_revision': source_revision(server_root),
        'effective_settings_verified': False,
        'settings_candidates': {},
        'custom_settings_files': sorted(p.name for p in (server_root / 'settings').glob('*.lua')),
        'live_server_verified': False,
    }
    for group, keys in SETTINGS.items():
        for relative in (f'settings/default/{group}.lua', f'settings/{group}.lua'):
            path = server_root / relative
            if path.is_file():
                report['settings_candidates'][relative] = safe_candidates(path.read_text(encoding='utf-8-sig'), keys)
    if dat_root is not None:
        dat_root = Path(dat_root).resolve(strict=True)
        report['dat_index_files'] = {}
        for name in ('VTABLE.DAT', 'FTABLE.DAT'):
            path = dat_root / name
            if not path.is_file():
                raise ValueError(f'The FFXI install is missing {name}.')
            report['dat_index_files'][name] = {'size_bytes': path.stat().st_size, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
    return report


def main():
    parser = argparse.ArgumentParser(description='Inspect selected LSB settings without executing Lua or contacting the server.')
    parser.add_argument('--server-root', type=Path, required=True)
    parser.add_argument('--dat-root', type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(inspect(args.server_root, args.dat_root), indent=2))
    except (OSError, ValueError) as exc:
        parser.exit(1, f'Inspection failed: {exc}\n')


if __name__ == '__main__':
    main()
