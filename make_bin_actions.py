import os
import zipfile

EXAMPLE_TARGET_FOLDER = "target/x86_64-unknown-linux-musl/release/examples"


def get_binary_names():
    return [f.split(".rs")[0] for f in os.listdir("wasm-json/examples/")]


def zip_functions(binary_names):

    for binary in binary_names:

        binary_path = f"{EXAMPLE_TARGET_FOLDER}/{binary}"

        with zipfile.ZipFile(f"{EXAMPLE_TARGET_FOLDER}/{binary}.zip", "w") as zip:
            zip.write(binary_path, arcname="exec")


if __name__ == "__main__":
    zip_functions(get_binary_names())