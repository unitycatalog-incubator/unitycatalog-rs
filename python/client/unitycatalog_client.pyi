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
