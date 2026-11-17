import io
import json
import os
import unittest
from unittest.mock import patch

from aerolith_playwright import AerolithPageMixin


class SdkTests(unittest.TestCase):
    def test_requests_include_shared_bearer_token(self):
        page = AerolithPageMixin(None)
        token = "test-only-secret-with-at-least-32-characters"
        with patch.dict(os.environ, {"AEROLITH_AGENT_TOKEN": token}):
            with patch("urllib.request.urlopen") as open_url:
                open_url.return_value.__enter__.return_value.read.return_value = b'{"success":true,"data":{"text":"hello"}}'
                self.assertEqual(page.get_text("#email"), "hello")
                request = open_url.call_args.args[0]
                self.assertEqual(request.get_header("Authorization"), f"Bearer {token}")
                self.assertIn("%23email", request.full_url)

    def test_command_errors_are_not_empty_results(self):
        body = io.BytesIO(json.dumps({"success": False, "error": "No target"}).encode())
        with self.assertRaisesRegex(RuntimeError, "No target"):
            AerolithPageMixin._check_response(body)

    def test_missing_token_fails_before_sending_request(self):
        with patch.dict(os.environ, {"AEROLITH_AGENT_TOKEN": ""}):
            with self.assertRaisesRegex(RuntimeError, "AEROLITH_AGENT_TOKEN"):
                AerolithPageMixin(None).semantic_snapshot()


if __name__ == "__main__":
    unittest.main()
