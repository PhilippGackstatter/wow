import os
import zipfile
import sys


def get_binary_names():
    return [f.split(".rs")[0] for f in os.listdir("ow-wasm-action/examples/")]


def zip_functions(binary_names):

    example_target_directory = sys.argv[1]

    for binary in binary_names:

        binary_path = f"{example_target_directory}/{binary}"

        with zipfile.ZipFile(f"{example_target_directory}/{binary}.zip", "w") as zip:
            zip.write(binary_path, arcname="exec")


if __name__ == "__main__":
    zip_functions(get_binary_names())