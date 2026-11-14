import json
import tempfile
import unittest
from pathlib import Path

from inspect_lsb import inspect, safe_candidates


class InspectionTests(unittest.TestCase):
    def test_only_allowlisted_scalar_values_leave_the_parser(self):
        text = """
        SQL_PASSWORD = 'private-secret',
        CLIENT_VER = '30261006_0', -- patch
        VER_LOCK = 2,
        LOGIN_AUTH_PORT = 54231,
        -- LOGIN_AUTH_PORT = 1,
        LOGIN_DATA_PORT = read_secret('private-secret'),
        """
        result = safe_candidates(text, ('CLIENT_VER', 'VER_LOCK', 'LOGIN_AUTH_PORT', 'LOGIN_DATA_PORT'))
        self.assertEqual(result, {'CLIENT_VER': ['30261006_0'], 'VER_LOCK': [2], 'LOGIN_AUTH_PORT': [54231], 'LOGIN_DATA_PORT': [None]})
        self.assertNotIn('private-secret', json.dumps(result))

    def test_preserves_ambiguous_assignments_without_claiming_effective_values(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            default = root / 'settings' / 'default'
            default.mkdir(parents=True)
            (default / 'network.lua').write_text('LOGIN_AUTH_PORT = 54231,\nSQL_PASSWORD = "private-secret",\n', encoding='utf-8')
            (root / 'settings' / 'network.lua').write_text('LOGIN_AUTH_PORT = 12345,\nLOGIN_AUTH_PORT = function_call(),\n', encoding='utf-8')
            (root / 'settings' / 'custom.lua').write_text('error("do not execute")', encoding='utf-8')
            result = inspect(root)
            self.assertEqual(result['settings_candidates']['settings/network.lua']['LOGIN_AUTH_PORT'], [12345, None])
            self.assertFalse(result['effective_settings_verified'])
            self.assertFalse(result['live_server_verified'])
            self.assertIn('custom.lua', result['custom_settings_files'])
            self.assertNotIn('private-secret', json.dumps(result))

    def test_missing_server_and_dat_roots_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaises(ValueError):
                inspect(root)
            (root / 'settings' / 'default').mkdir(parents=True)
            with self.assertRaises(ValueError):
                inspect(root, root)

    def test_dat_indexes_are_hashed_and_files_are_unchanged(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'settings' / 'default').mkdir(parents=True)
            for name in ('VTABLE.DAT', 'FTABLE.DAT'):
                (root / name).write_bytes(b'abc')
            before = {p.relative_to(root): p.read_bytes() for p in root.rglob('*') if p.is_file()}
            result = inspect(root, root)
            self.assertEqual(result['dat_index_files']['VTABLE.DAT']['sha256'], 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad')
            after = {p.relative_to(root): p.read_bytes() for p in root.rglob('*') if p.is_file()}
            self.assertEqual(before, after)


if __name__ == '__main__':
    unittest.main()
