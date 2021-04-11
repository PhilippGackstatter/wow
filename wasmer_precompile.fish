#!/usr/bin/fish

function module_basename --argument path
    set mod_name (basename -s .wasm $path)
    echo $mod_name
end

for arg in $argv
    echo $arg
    set mod_name (module_basename $arg)
    set dir_name (dirname $arg)
    # --target aarch64-unknown-linux
    wasmer compile --target aarch64-unknown-linux --llvm --native $arg -o "$mod_name.wasmer"
    zip -m "$dir_name/$mod_name-wasmer.zip" "$mod_name.wasmer"
end
