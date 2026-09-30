import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

from corpus import execute_check


@unittest.skipUnless(os.name == 'posix', 'runner uses POSIX process groups')
class TimeoutTests(unittest.TestCase):
    def test_success_and_failure_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            for status in (0, 3):
                with (Path(directory) / 'output.log').open('w') as log:
                    result = execute_check([sys.executable, '-c', f'raise SystemExit({status})'],
                                           directory, os.environ.copy(), log, 5)
                self.assertEqual(result, (status, False))

    def test_timeout_stops_child_processes(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / 'survived'
            child = f'import time; from pathlib import Path; time.sleep(0.8); Path({str(marker)!r}).touch()'
            parent = f'import subprocess,sys,time; subprocess.Popen([sys.executable,"-c",{child!r}]); time.sleep(30)'
            with (Path(directory) / 'output.log').open('w') as log:
                code, timed_out = execute_check([sys.executable, '-c', parent],
                                               directory, os.environ.copy(), log, 0.3)
            self.assertTrue(timed_out)
            self.assertNotEqual(code, 0)
            time.sleep(1)
            self.assertFalse(marker.exists(), 'descendant survived timeout')
