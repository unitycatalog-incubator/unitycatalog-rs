import { FlatTree } from "@fluentui/react-components";
import CatalogTree from "./catalog/CatalogTree";
import CredentialTree from "./credentials/CredentialTree";
import ExternalLocationTree from "./external_locations/ExternalLocationTree";
import RecipientTree from "./recipients/RecipientTree";
import ShareTree from "./shares/ShareTree";
import { TreeProvider } from "../context";

export const TreeView = () => {
  return (
    <FlatTree appearance="subtle">
      <TreeProvider value={["catalogs"]}>
        <CatalogTree setSize={5} setPos={1} />
      </TreeProvider>
      <TreeProvider value={["credentials"]}>
        <CredentialTree setSize={5} setPos={2} />
      </TreeProvider>
      <TreeProvider value={["external_locations"]}>
        <ExternalLocationTree setSize={5} setPos={3} />
      </TreeProvider>
      <TreeProvider value={["recipients"]}>
        <RecipientTree setSize={5} setPos={4} />
      </TreeProvider>
      <TreeProvider value={["shares"]}>
        <ShareTree setSize={5} setPos={5} />
      </TreeProvider>
    </FlatTree>
  );
};

export default TreeView;
