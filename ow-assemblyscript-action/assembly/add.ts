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