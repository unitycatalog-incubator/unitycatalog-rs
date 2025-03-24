import { useCallback, useState, useEffect, useMemo } from "react";
import ucClient, {
  UpdateShareRequest,
  DataObject,
  DataObjectUpdate,
} from "../../client";
import { InputChange } from "../../types";
import { CreateFormState, Input, CreateResource } from "../forms";
import { useQuery } from "@tanstack/react-query";
import { useExplorer } from "../../context";
import {
  DataGridBody,
  DataGridRow,
  DataGrid,
  DataGridHeader,
  DataGridHeaderCell,
  DataGridCell,
  TableCellLayout,
  TableColumnDefinition,
  createTableColumn,
  Tag,
  Button,
  Divider,
  Text,
  makeStyles,
  tokens,
} from "@fluentui/react-components";
import {
  DeleteRegular,
  EditRegular,
  ArrowHookUpLeftRegular,
} from "@fluentui/react-icons";
import CatalogSelect, { SelectedItem } from "../catalog/CatalogSelect";
import { ActionJson } from "../../gen/unitycatalog/shares/v1/svc_pb";

const useStyles = makeStyles({
  deleted: {
    color: tokens.colorStatusDangerForeground3,
  },
  added: {
    color: tokens.colorStatusSuccessForeground3,
  },
  updated: {
    color: tokens.colorStatusWarningForeground3,
  },
});

type DataObjectRow = Required<DataObjectUpdate> & {
  dataObject: Required<
    Pick<DataObject, "name" | "dataObjectType" | "sharedAs">
  > &
    DataObject;
};

function getColumns(
  selectClassName: (action: ActionJson) => string | undefined,
  onRemove: (name: string) => void,
  onRestore: (name: string) => void,
): TableColumnDefinition<DataObjectRow>[] {
  return [
    createTableColumn<DataObjectRow>({
      columnId: "name",
      compare: (a, b) => {
        return a.dataObject.name.localeCompare(b.dataObject.name);
      },
      renderHeaderCell: () => {
        return "Name";
      },
      renderCell: (item) => {
        return (
          <TableCellLayout>
            <Text
              className={selectClassName(item.action)}
              strikethrough={item.action === "REMOVE"}
            >
              {item.dataObject.name}
            </Text>
          </TableCellLayout>
        );
      },
    }),
    createTableColumn<DataObjectRow>({
      columnId: "dataObjectType",
      compare: (a, b) => {
        return a.dataObject.dataObjectType.localeCompare(
          b.dataObject.dataObjectType,
        );
      },
      renderHeaderCell: () => {
        return "Type";
      },
      renderCell: (item) => {
        return (
          <TableCellLayout className={selectClassName(item.action)}>
            <Text
              className={selectClassName(item.action)}
              strikethrough={item.action === "REMOVE"}
            >
              {item.dataObject.dataObjectType}
            </Text>
          </TableCellLayout>
        );
      },
    }),
    createTableColumn<DataObjectRow>({
      columnId: "sharedAs",
      compare: (a, b) => {
        return a.dataObject.sharedAs.localeCompare(b.dataObject.sharedAs);
      },
      renderHeaderCell: () => {
        return "Shared As";
      },
      renderCell: (item) => {
        return (
          <Tag size="small" className={selectClassName(item.action)}>
            <Text
              className={selectClassName(item.action)}
              strikethrough={item.action === "REMOVE"}
              size={200}
            >
              {item.dataObject.sharedAs}
            </Text>
          </Tag>
        );
      },
    }),
    createTableColumn<DataObjectRow>({
      columnId: "action",
      renderHeaderCell: () => {
        return "Action";
      },
      renderCell: (item) => {
        return (
          <span style={{ display: "flex", columnGap: 5 }}>
            {item.action !== "REMOVE" ? (
              <>
                <Button
                  icon={<DeleteRegular />}
                  onClick={() => onRemove(item.dataObject.name)}
                />
                <Button icon={<EditRegular />} />
              </>
            ) : (
              <Button
                icon={<ArrowHookUpLeftRegular />}
                onClick={() => onRestore(item.dataObject.name)}
              />
            )}
          </span>
        );
      },
    }),
  ];
}

function selectedItemToRow(item: SelectedItem): DataObjectRow {
  return {
    action: "ADD",
    dataObject: {
      name: item.fullName,
      dataObjectType: item.kind === "schema" ? "SCHEMA" : "TABLE",
      // remove the first part (catalog) of the name, as this will always
      // the name of the share when exposed via delta sharing.
      sharedAs: item.fullName.split(".").slice(1).join("."),
      partitions: [],
      historyDataSharingStatus: "DISABLED",
      enableCdf: false,
    },
  } as DataObjectRow;
}

function UpdateShareForm({
  values,
  setValues,
}: CreateFormState<UpdateShareRequest>) {
  const styles = useStyles();
  // @ts-expect-error
  const [updates, setUpdates] = useState<DataObjectRow[]>(values.updates ?? []);

  const onAdd = useCallback((selected: SelectedItem[]) => {
    setUpdates((curr) => [...curr, ...selected.map(selectedItemToRow)]);
  }, []);

  const onRemove = useCallback((name: string) => {
    setUpdates((curr) =>
      curr
        // if the item was just added, remove it from the list
        .filter(
          (item) => item.dataObject.name !== name || item.action !== "ADD",
        )
        // otherwise mark it for removal
        .map((item) =>
          item.dataObject.name === name ? { ...item, action: "REMOVE" } : item,
        ),
    );
  }, []);

  const onRestore = useCallback((name: string) => {
    setUpdates((curr) =>
      curr.map((item) =>
        item.dataObject.name === name
          ? { ...item, action: "ACTION_UNSPECIFIED" }
          : item,
      ),
    );
  }, []);

  const selectClassName = useCallback((action: ActionJson) => {
    switch (action) {
      case "ADD":
        return styles.added;
      case "REMOVE":
        return styles.deleted;
      case "UPDATE":
        return styles.updated;
      default:
        return undefined;
    }
  }, []);

  const { scope: queryKey } = useExplorer();
  const { data, status } = useQuery({
    queryKey: ["get", ...(queryKey ?? [])],
    queryFn: ({ queryKey }) =>
      ucClient.shares.get({ name: queryKey[queryKey.length - 1] }),
  });

  const onNameChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, newName: data.value }));
  }, []);
  const onCommentChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, comment: data.value }));
  }, []);

  const columns = useMemo(
    () => getColumns(selectClassName, onRemove, onRestore),
    [selectClassName, onRemove, onRestore],
  );

  // seet the initial list of data objects to the share's data objects
  useEffect(() => {
    if (status === "success") {
      if (values.name !== data.name) {
        setValues((curr) => ({
          ...curr,
          name: data.name,
          comment: data.comment,
        }));
        setUpdates(
          (data.dataObjects ?? []).map(
            (dataObject) =>
              ({
                action: "ACTION_UNSPECIFIED",
                dataObject,
              }) as DataObjectRow,
          ),
        );
      } else {
        // the values only contain the changes items, so we need to merge
        // the unchanged items from the data object
        const changedItems = (values.updates ?? []).map(
          (item) => item.dataObject?.name,
        );
        const updates = (data.dataObjects ?? [])
          .filter((dataObject) => !changedItems.includes(dataObject.name))
          .map(
            (dataObject) =>
              ({
                action: "ACTION_UNSPECIFIED",
                dataObject,
              }) as DataObjectRow,
          )
          // @ts-expect-error
          .concat(values.updates ?? []);
        setUpdates(updates);
      }
    }
  }, [data, status, values]);

  // update data object changes in the update request object
  // unchanged data objects are not included in the request
  useEffect(() => {
    setValues((curr) => ({
      ...curr,
      updates: updates.filter((item) => item.action !== "ACTION_UNSPECIFIED"),
    }));
  }, [updates]);

  return (
    <>
      <Input
        label="Name"
        value={values.newName ?? ""}
        onChange={onNameChange}
        placeholder={values.name}
      />
      <Input
        label="Comment"
        onChange={onCommentChange}
        value={values.comment ?? ""}
      />
      <div style={{ paddingTop: 15, paddingBottom: 10 }}>
        <span style={{ display: "flex", justifyContent: "space-between" }}>
          <Text size={500}>Shared Objects</Text>
          <CatalogSelect selectionMode="single" onAdd={onAdd} />
        </span>
        <Divider />
      </div>
      <DataGrid items={updates} columns={columns} style={{ minWidth: "550px" }}>
        <DataGridHeader>
          <DataGridRow>
            {({ renderHeaderCell }) => (
              <DataGridHeaderCell>{renderHeaderCell()}</DataGridHeaderCell>
            )}
          </DataGridRow>
        </DataGridHeader>
        <DataGridBody<DataObjectRow>>
          {({ item, rowId }) => (
            <DataGridRow<DataObjectRow> key={rowId}>
              {({ renderCell }) => (
                <DataGridCell>{renderCell(item)}</DataGridCell>
              )}
            </DataGridRow>
          )}
        </DataGridBody>
      </DataGrid>
    </>
  );
}

function UpdateShare() {
  return (
    <CreateResource
      createFn={ucClient.shares.update}
      FormComponent={UpdateShareForm}
      resourceType="share"
      defaultValues={{}}
      typeName="UpdateShareRequest"
      operation="update"
    />
  );
}

export default UpdateShare;
