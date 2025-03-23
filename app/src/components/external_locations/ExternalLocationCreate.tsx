import {
  Field,
  makeStyles,
  tokens,
  Checkbox,
  Option,
  Dropdown,
} from "@fluentui/react-components";
import { useCallback } from "react";
import { CreateResource, CreateFormState, Input } from "../forms";
import ucClient, { CreateExternalLocationRequest } from "../../client";
import { CheckboxChange, InputChange, DropdownSelect } from "../../types";
import { useQuery } from "@tanstack/react-query";

const useStyles = makeStyles({
  root: {
    display: "flex",
    height: "100%",
    width: "100%",
    flexDirection: "column",
    overflowY: "scroll",
  },

  toolbar: {
    justifyContent: "space-between",
    borderBottomColor: tokens.colorNeutralForeground4,
    borderBottomWidth: "1px",
    borderBottomStyle: "solid",
  },

  content: {
    flex: 1,
    padding: "25px 25px 10px 25px",
    display: "flex",
    flexDirection: "column",
    rowGap: "10px",
    overflowY: "auto",
  },

  line: {
    display: "flex",
    flexDirection: "row",
    gap: "10px",
  },
});

function ExternalLocationForm({
  values,
  setValues,
}: CreateFormState<CreateExternalLocationRequest>) {
  const styles = useStyles();

  const { data, status } = useQuery({
    queryKey: ["credentials"],
    queryFn: () => ucClient.credentials.list(),
    refetchInterval: 30000,
  });

  const onNameChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, name: data.value }));
  }, []);
  const onCredentialNameChange: DropdownSelect = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, credentialName: data.optionValue }));
  }, []);
  const onUrlChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, url: data.value }));
  }, []);
  const onCommentChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, comment: data.value }));
  }, []);
  const onReadOnlyChange: CheckboxChange = useCallback((_ev, data) => {
    setValues((curr) => ({
      ...curr,
      readOnly: data.checked === "mixed" ? undefined : data.checked,
    }));
  }, []);
  const onSkipValidationChange: CheckboxChange = useCallback((_ev, data) => {
    setValues((curr) => ({
      ...curr,
      skipValidation: data.checked === "mixed" ? undefined : data.checked,
    }));
  }, []);

  return (
    <>
      <div className={styles.line}>
        <Input
          label="Name"
          style={{ flex: "1" }}
          value={values.name ?? ""}
          onChange={onNameChange}
        />
        <Field label="Credential name" style={{ flex: "1" }}>
          <Dropdown
            onOptionSelect={onCredentialNameChange}
            value={values.credentialName ?? ""}
          >
            {status === "success" &&
              data.map(
                (c) =>
                  c.name && (
                    <Option key={c.name} value={c.name}>
                      {c.name}
                    </Option>
                  ),
              )}
          </Dropdown>
        </Field>
        <Field label="Properties">
          <div className={styles.line}>
            <Checkbox
              label={"read only"}
              checked={values.readOnly ?? "mixed"}
              onChange={onReadOnlyChange}
            />
            {false && (
              <Checkbox
                label={"skip validation"}
                checked={values.skipValidation ?? "mixed"}
                onChange={onSkipValidationChange}
              />
            )}
          </div>
        </Field>
      </div>
      <Input
        label="Url"
        onChange={onUrlChange}
        value={values.url ?? ""}
        type="url"
      />
      <Input
        label="Comment"
        onChange={onCommentChange}
        value={values.comment ?? ""}
      />
    </>
  );
}

function CreateExternalLocation() {
  return (
    <CreateResource
      createFn={ucClient.externalLocations.create}
      FormComponent={ExternalLocationForm}
      resourceType="external location"
      defaultValues={{ readOnly: false, skipValidation: false }}
      typeName="CreateExternalLocationRequest"
    />
  );
}

export default CreateExternalLocation;
