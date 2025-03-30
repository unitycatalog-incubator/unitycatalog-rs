import { useCallback } from "react";
import ucClient, {
  CreateTableRequest,
  DataSourceFormat,
  TableType,
} from "../../client";
import { InputChange, TabSelect, DropdownSelect } from "../../types";
import { CreateResource, CreateFormState, Input } from "../forms";
import { useExplorer } from "../../context";
import {
  makeStyles,
  Tab,
  TabList,
  tokens,
  Dropdown,
  Option,
  Field,
} from "@fluentui/react-components";

const useStyles = makeStyles({
  tabs: {
    padding: "10px 0 10px 10px",
    display: "flex",
    flexDirection: "column",
    rowGap: tokens.spacingVerticalL,
  },
});

function TableForm({ values, setValues }: CreateFormState<CreateTableRequest>) {
  const styles = useStyles();

  const onTabSelect: TabSelect = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, tableType: data.value as TableType }));
  }, []);
  const onNameChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, name: data.value }));
  }, []);
  const onCommentChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, comment: data.value }));
  }, []);
  const onStorageChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, storageLocation: data.value }));
  }, []);
  const onFormatSelect: DropdownSelect = useCallback((_ev, data) => {
    setValues((curr) => ({
      ...curr,
      dataSourceFormat: data.optionValue as DataSourceFormat,
    }));
  }, []);

  return (
    <>
      <span style={{ display: "flex" }}>
        <Input
          label="Name"
          value={values.name ?? ""}
          onChange={onNameChange}
          style={{ flex: 1 }}
        />
      </span>
      <Input
        label="Comment"
        onChange={onCommentChange}
        value={values.comment ?? ""}
      />
      <TabList selectedValue={values.tableType} onTabSelect={onTabSelect}>
        <Tab value="EXTERNAL">Managed</Tab>
        <Tab value="MANAGED">Sharing</Tab>
      </TabList>
      <div className={styles.tabs}>
        {values.tableType === "EXTERNAL" && (
          <>
            <Input
              label="Storage root"
              value={values.storageLocation ?? ""}
              onChange={onStorageChange}
              type="url"
            />
            <Field label="Source format">
              <Dropdown
                value={values.dataSourceFormat ?? "DELTA"}
                onOptionSelect={onFormatSelect}
              >
                <Option value="DELTA">Delta</Option>
                <Option value="PARQUET">Parquet</Option>
              </Dropdown>
            </Field>
          </>
        )}
      </div>
    </>
  );
}

function CreateTable() {
  const { scope } = useExplorer();

  if (!scope) {
    return "No scope selected";
  }

  if (scope.length !== 3 || scope[0] !== "catalogs") {
    return "Invalid scope";
  }

  return (
    <CreateResource
      createFn={ucClient.tables.create}
      FormComponent={TableForm}
      resourceType="table"
      defaultValues={{
        catalogName: scope[1],
        schemaName: scope[2],
        properties: {},
        dataSourceFormat: "DELTA",
        tableType: "EXTERNAL",
      }}
      typeName="CreateTableRequest"
    />
  );
}

export default CreateTable;
