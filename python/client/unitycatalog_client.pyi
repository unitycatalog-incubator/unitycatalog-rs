class CatalogInfo:
    id: str | None
    name: str
    owner: str | None
    comment: str | None
    properties: dict | None
    storage_root: str | None
    provider_name: str | None
    share_name: str | None
    catalog_type: int | None
    created_at: int | None
    created_by: str | None
    updated_at: int | None
    updated_by: str | None
    browse_only: bool | None


class CreateCatalogRequest:
    name: str
    comment: str | None
    storage_root: str | None
    provider_name: str | None
    share_name: str | None


    def __init__(
        self,
        name: str,
        comment: str | None = None,
        storage_root: str | None = None,
        provider_name: str | None = None,
        share_name: str | None = None,
    ) -> None: ...

class CatalogClient:
    def get(self) -> CatalogInfo: ...


class UnityCatalogClient:
    def __init__(self, base_url: str) -> None: ...

    def catalogs(self, name: str) -> CatalogClient: ...
