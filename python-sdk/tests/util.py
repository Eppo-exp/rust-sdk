import time
import os
import json

import eppo_client
from eppo_client.config import Config, AssignmentLogger


def init(suite, *, wait_for_init=True, assignment_logger=AssignmentLogger()):
    client = eppo_client.init(
        Config(
            api_key="blah",
            base_url=f"http://localhost:8378/{suite}/api",
            assignment_logger=assignment_logger,
        )
    )
    if wait_for_init:
        wait_for_initialization()
    return client


def wait_for_initialization():
    client = eppo_client.get_instance()
    if not client.is_initialized():
        if hasattr(client, "wait_for_initialization"):
            client.wait_for_initialization()
        else:
            time.sleep(0.1)


def load_test_files(dir):
    TEST_DIR = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "../../sdk-test-data/ufc/", dir
    )
    test_data = []
    for file_name in os.listdir(TEST_DIR):
        # dynamic-typing tests allow passing arbitrary/invalid values to
        # ContextAttributes. Our implementation is more strongly typed and
        # checks that attributes have proper types and throws TypeError
        # otherwise (at ContextAttributes construction, not
        # evaluation). Therefore, these tests are not applicable.
        if not file_name.endswith(".dynamic-typing.json"):
            with open("{}/{}".format(TEST_DIR, file_name)) as test_case_json:
                test_case_dict = json.load(test_case_json)
                test_case_dict["file_name"] = file_name
                test_data.append(test_case_dict)
    return test_data
