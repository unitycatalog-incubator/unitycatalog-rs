import {
  DrawerBody,
  DrawerHeader,
  DrawerHeaderTitle,
  InlineDrawer,
  makeStyles,
  tokens,
} from "@fluentui/react-components";
import TreeView from "./TreeView";
import CreateSchema from "./catalog/SchemaCreate";
import CreateTable from "./catalog/TableCreate";
import CreateCatalog from "./catalog/CatalogCreate";
import CreateCredential from "./credentials/CredentialCreate";
import CreateExternalLocation from "./external_locations/ExternalLocationCreate";
import CreateRecipient from "./recipients/RecipientCreate";
import CreateShare from "./shares/ShareCreate";
import UpdateShare from "./shares/ShareUpdate";
import { ExplorerProvider, ExplorerPropsInner, useExplorer } from "../context";
import { useState, useMemo } from "react";
import { ViewResource, CreateResource } from "./forms";
import ucClient from "../client";
import * as ct from "../client";

const useStyles = makeStyles({
  root: {
    display: "flex",
    height: "100%",
    width: "100%",
    userSelect: "auto",
  },

  container: {
    position: "relative",
  },

  drawer: {
    width: "320px",
    borderRightColor: tokens.colorNeutralForeground4,
    borderRightWidth: "1px",
    borderRightStyle: "solid",
    height: "100%",
  },

  content: {
    flex: "1",
  },
});

type OperationBase<Resource, Op, Request, Info> = {
  resource: Resource;
  operation: Op;
  op: (request: Request) => Promise<Info>;
  requestSchemaName: string;
  defaults?: Partial<Request>;
};

type Operation =
  | OperationBase<"catalog", "create", ct.CreateCatalogRequest, ct.CatalogInfo>
  | OperationBase<"catalog", "update", ct.UpdateCatalogRequest, ct.CatalogInfo>
  | OperationBase<
      "credential",
      "create",
      ct.CreateCredentialRequest,
      ct.CredentialInfo
    >
  | OperationBase<
      "credential",
      "update",
      ct.UpdateCredentialRequest,
      ct.CredentialInfo
    >
  | OperationBase<
      "external_location",
      "create",
      ct.CreateExternalLocationRequest,
      ct.ExternalLocationInfo
    >
  | OperationBase<
      "external_location",
      "update",
      ct.UpdateExternalLocationRequest,
      ct.ExternalLocationInfo
    >
  | OperationBase<
      "recipient",
      "create",
      ct.CreateRecipientRequest,
      ct.RecipientInfo
    >
  | OperationBase<
      "recipient",
      "update",
      ct.UpdateRecipientRequest,
      ct.RecipientInfo
    >
  | OperationBase<"share", "create", ct.CreateShareRequest, ct.ShareInfo>
  | OperationBase<"share", "update", ct.UpdateShareRequest, ct.ShareInfo>
  | OperationBase<"schema", "create", ct.CreateSchemaRequest, ct.SchemaInfo>
  | OperationBase<"schema", "update", ct.UpdateSchemaRequest, ct.SchemaInfo>
  | OperationBase<"table", "create", ct.CreateTableRequest, ct.TableInfo>;

function useOperation(): Operation | undefined {
  const { display, scope } = useExplorer();

  if (!display || !scope) {
    throw new Error(
      "useOperation should not be called without display and scope",
    );
  }

  if (display === "view") {
    throw new Error("useOperation should not be called in view mode");
  }

  return useMemo(() => {
    let result: Operation | undefined;

    switch (display) {
      case "create":
        switch (scope.length) {
          case 1:
            switch (scope[0]) {
              case "catalogs":
                return {
                  resource: "catalog",
                  operation: "create",
                  op: ucClient.catalogs.create,
                  requestSchemaName: "CreateCatalogRequest",
                } as Operation;
              case "credentials":
                return {
                  resource: "credential",
                  operation: "create",
                  op: ucClient.credentials.create,
                  requestSchemaName: "CreateCredentialRequest",
                } as Operation;
              case "external_locations":
                return {
                  resource: "external_location",
                  operation: "create",
                  op: ucClient.externalLocations.create,
                  requestSchemaName: "CreateExternalLocationRequest",
                } as Operation;
              case "recipients":
                return {
                  resource: "recipient",
                  operation: "create",
                  op: ucClient.recipients.create,
                  requestSchemaName: "CreateRecipientRequest",
                } as Operation;
              case "shares":
                return {
                  resource: "share",
                  operation: "create",
                  op: ucClient.shares.create,
                  requestSchemaName: "CreateShareRequest",
                } as Operation;
              default:
                break;
            }
            break;
          case 2:
            if (scope[0] === "catalogs") {
              return {
                resource: "schema",
                operation: "create",
                op: ucClient.schemas.create,
                requestSchemaName: "CreateSchemaRequest",
                defaults: { catalogName: scope[1] },
              } as Operation;
            }
            break;
          case 3:
            if (scope[0] === "catalogs") {
              return {
                resource: "table",
                operation: "create",
                op: ucClient.tables.create,
                requestSchemaName: "CreateTableRequest",
                defaults: { catalogName: scope[1], schemaName: scope[2] },
              } as Operation;
            }
            break;
          default:
            break;
        }
        break;
      case "edit":
        switch (scope.length) {
          case 2:
            switch (scope[0]) {
              case "catalogs":
                return {
                  resource: "catalog",
                  operation: "update",
                  op: ucClient.catalogs.update,
                  requestSchemaName: "UpdateCatalogRequest",
                  defaults: { name: scope[1] },
                } as Operation;
              case "credentials":
                return {
                  resource: "credential",
                  operation: "update",
                  op: ucClient.credentials.update,
                  requestSchemaName: "UpdateCredentialRequest",
                  defaults: { name: scope[1] },
                } as Operation;
              case "external_locations":
                return {
                  resource: "external_location",
                  operation: "update",
                  op: ucClient.externalLocations.update,
                  requestSchemaName: "UpdateExternalLocationRequest",
                  defaults: { name: scope[1] },
                } as Operation;
              case "recipients":
                return {
                  resource: "recipient",
                  operation: "update",
                  op: ucClient.recipients.update,
                  requestSchemaName: "UpdateRecipientRequest",
                  defaults: { name: scope[1] },
                } as Operation;
              case "shares":
                return {
                  resource: "share",
                  operation: "update",
                  op: ucClient.shares.update,
                  requestSchemaName: "UpdateShareRequest",
                  defaults: { name: scope[1] },
                } as Operation;
              default:
                break;
            }
            break;
          case 3:
            if (scope[0] === "catalogs") {
              return {
                resource: "schema",
                operation: "update",
                op: ucClient.schemas.update,
                requestSchemaName: "UpdateSchemaRequest",
                defaults: { fullName: `${scope[0]}.${scope[1]}` },
              } as Operation;
            }
            break;
          default:
            break;
        }
        break;
      default:
        break;
    }
  }, [display, scope]);
}

function GenericUpdate() {
  const result = useOperation();

  if (!result) {
    return null;
  }

  const { op, requestSchemaName, defaults, operation, resource } = result;
  return (
    <CreateResource
      createFn={op}
      resourceType={resource}
      defaultValues={defaults ?? {}}
      typeName={requestSchemaName}
      operation={operation}
    />
  );
}

function ExplorerContent() {
  const { display, scope } = useExplorer();

  switch (display) {
    case "create":
      if (scope?.length === 1) {
        if (scope[0] === "catalogs") {
          return <CreateCatalog />;
        } else if (scope[0] === "credentials") {
          return <CreateCredential />;
        } else if (scope[0] === "external_locations") {
          return <CreateExternalLocation />;
        } else if (scope[0] === "recipients") {
          return <CreateRecipient />;
        } else if (scope[0] === "shares") {
          return <CreateShare />;
        }
      } else if (scope && scope[0] === "catalogs") {
        if (scope?.length === 2) {
          return <CreateSchema />;
        } else if (scope?.length === 3) {
          return <CreateTable />;
        }
      }
      break;
    case "edit":
      if (scope?.length === 2) {
        if (scope[0] === "shares") {
          return <UpdateShare />;
        }
      }
      return <GenericUpdate />;
    case "view":
      return <ViewResource />;
    default:
      break;
  }

  return "no content";
}

function Explorer() {
  const styles = useStyles();
  const [state, setState] = useState<ExplorerPropsInner>({});

  return (
    <ExplorerProvider value={{ ...state, update: setState }}>
      <div className={styles.root}>
        <div className={styles.container}>
          <InlineDrawer open className={styles.drawer}>
            <DrawerHeader>
              <DrawerHeaderTitle>Catalog Browser</DrawerHeaderTitle>
            </DrawerHeader>
            <DrawerBody>
              <TreeView />
            </DrawerBody>
          </InlineDrawer>
        </div>
        <div className={styles.content}>
          <ExplorerContent />
        </div>
      </div>
    </ExplorerProvider>
  );
}

export default Explorer;
