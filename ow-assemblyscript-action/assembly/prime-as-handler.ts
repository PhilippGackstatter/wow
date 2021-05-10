import { JSON } from "assemblyscript-json";

export default function main(): void {
    let json = new JSON.Obj()
    json.set("upperBound", 200000);
    let result = handler(json)
}

export function handler(json: JSON.Obj): JSON.Obj | null {

    let upperBound = json.getInteger("upperBound")

    if (upperBound != null) {
        let num_primes = eratosthenes(upperBound.valueOf() as i32)
        let result = new JSON.Obj()
        result.set("result", num_primes)
        return result
    }

    return null
}

// https://rosettacode.org/wiki/Sieve_of_Eratosthenes#JavaScript
export function eratosthenes(limit: i32): i32 {
    var prms: number[] = [];

    if (limit >= 2) prms = [2];
    if (limit >= 3) {
        let sqrtlmt = (Math.sqrt(limit as f64) - 3) as i32 >> 1;
        let lmt = (limit - 3) >> 1;
        let bfsz = (lmt >> 5) + 1
        let buf = [];
        for (let i = 0; i < bfsz; i++)
            buf.push(0);
        for (let i = 0; i <= sqrtlmt; i++)
            if ((buf[i >> 5] & (1 << (i & 31))) == 0) {
                let p = i + i + 3;
                for (let j = (p * p - 3) >> 1; j <= lmt; j += p)
                    buf[j >> 5] |= 1 << (j & 31);
            }
        for (let i = 0; i <= lmt; i++)
            if ((buf[i >> 5] & (1 << (i & 31))) == 0)
                prms.push(i + i + 3);
    }

    return prms.length;
}