import { setupServer } from "msw/node";
import { handlers } from "../__mocks__/handlers";
import { UnityCatalogClient } from "../unitycatalog/client";

const server = setupServer(...handlers);

describe("UnityCatalogClient", () => {
  beforeAll(() => {
    server.listen();
  });

  afterEach(() => {
    server.resetHandlers();
  });

  afterAll(() => {
    server.close();
  });

  it("should create a new instance", () => {
    const client = new UnityCatalogClient(
      "http://unitycatalog.io/api/2.1/unity-catalog/",
    );
    expect(client).toBeDefined();
  });
});
