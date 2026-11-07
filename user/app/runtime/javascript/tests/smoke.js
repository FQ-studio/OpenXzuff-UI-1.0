const values = [1, 2, 3, 4];
const result = values.map((value) => value * 2).join(",");
if (result !== "2,4,6,8") {
    throw new Error("array/map smoke test failed");
}
print("Xzuff JS host smoke: PASS");
print("ECMAScript result:", result);
