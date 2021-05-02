import { JSON } from "assemblyscript-json";

function handler(json: JSON.Obj): JSON.Obj | null {

  let param1 = json.getInteger("param1")
  let param2 = json.getInteger("param2")

  if (param1 != null && param2 != null) {

    let result = new JSON.Obj()
    result.set("result", param1.valueOf() + param2.valueOf())
    return result

  } else {
    console.error("parameters `param1` or `param2` were not present")
    return null
  }

}

export default function _start(): void {

  let len = parseInt(process.argv[process.argv.length - 1]) as i32

  let jsonParams = loadJson(len)

  let result = handler(jsonParams)

  if (result != null) {
    storeJson(result, false)
  } else {
    let error = new JSON.Obj()
    error.set("message", "error")
    storeJson(error, true)
  }
}

function loadJson(len: i32): JSON.Obj {
  let buf = new ArrayBuffer(len);
  let buf_u8 = Uint8Array.wrap(buf)

  for (let i: i32 = 0; i < len; i++) {
    buf_u8[i] = load<u8>(buffer_ptr + i) as u8
  }

  const json_str = String.UTF8.decode(buf, false)

  console.log(`Read ${json_str}`)

  let jsonObj: JSON.Obj = <JSON.Obj>(JSON.parse(json_str));

  return jsonObj
}

// , error?: bool | undefined
function storeJson(json: JSON.Obj, error: boolean): void {
  console.log(`Storing json`)

  // Wrap it in the Rust Result type
  let result_wrapper = new JSON.Obj()
  if (error) {
    result_wrapper.set("Err", json)
  } else {
    result_wrapper.set("Ok", json)
  }

  let json_str: string = result_wrapper.toString();

  const stringBuffer = String.UTF8.encode(json_str, false);
  const stringBufferBytes = Uint8Array.wrap(stringBuffer);

  wasm_memory_buffer_allocate_space(stringBufferBytes.byteLength)
  buffer_size = stringBufferBytes.byteLength

  for (var i = 0; i < stringBufferBytes.byteLength; i++) {
    store<u8>(buffer_ptr + i, stringBufferBytes[i])
  }
}

var buffer_size: i32;
var buffer_ptr: usize;

export function wasm_memory_buffer_allocate_space(size: i32): void {
  let buffer = new Array<u8>(size)
  buffer_ptr = memory.data(8)
  store<Array<u8>>(buffer_ptr, buffer);
}

export function get_wasm_memory_buffer_len(): usize {
  return buffer_size
}

export function get_wasm_memory_buffer_pointer(): usize {
  return buffer_ptr
}

export function abort(message: string | null, fileName: string | null, lineNumber: u32, columnNumber: u32): void { }
