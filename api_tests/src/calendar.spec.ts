import axios from "axios";

describe("it works", () => {
  it("should give status 200", async () => {
    let status = await axios.get("http://localhost:5000");
    expect(status.status).toBe(200);
  });
});
