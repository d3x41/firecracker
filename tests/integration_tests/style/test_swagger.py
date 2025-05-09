# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests ensuring codebase style compliance for the OpenAPI specification."""

from pathlib import Path

from openapi_spec_validator import validate
from openapi_spec_validator.readers import read_from_filename


def validate_swagger(swagger_spec):
    """Fail if OpenAPI spec is not followed."""
    spec_dict, _ = read_from_filename(swagger_spec)
    validate(spec_dict)


def test_firecracker_swagger():
    """
    Test that Firecracker swagger specification is valid.
    """
    swagger_spec = Path("../src/firecracker/swagger/firecracker.yaml")
    validate_swagger(swagger_spec)
