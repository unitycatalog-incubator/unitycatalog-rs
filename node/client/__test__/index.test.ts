import { UnityCatalogClient } from "../unitycatalog/client";
import { setupServer } from 'msw/node'
import { handlers } from '../__mocks__/handlers'

const server = setupServer(...handlers)

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

  it("should create a new instance", async () => {
    let client = new UnityCatalogClient("http://unitycatalog.io/api/2.1/unity-catalog/");
    let catalogs = await client.listCatalogs();
    expect(catalogs.length).toEqual(1);
  });
});
