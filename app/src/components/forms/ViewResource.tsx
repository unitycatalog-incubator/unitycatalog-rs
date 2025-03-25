import {
  makeStyles,
  Toolbar,
  ToolbarButton,
  tokens,
  Text,
  ToolbarGroup,
  ToolbarToggleButton,
  ToolbarProps,
} from "@fluentui/react-components";
import {
  useCallback,
  useState,
  useRef,
  useEffect,
  useMemo,
  ComponentType,
} from "react";
import { useExplorer, useTreeFunctions } from "../../context";
import {
  ArrowLeftRegular,
  BracesRegular,
  EditRegular,
} from "@fluentui/react-icons";
import type monaco from "monaco-editor";
import JsonEditor from "./editor/JsonEditor";
import { OnMount } from "@monaco-editor/react";
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
    backgroundColor: tokens.colorNeutralBackground2,
  },

  editor: {
    flex: 1,
  },

  editorHidden: {
    display: "none",
  },
});

export type ViewFormState<T> = {
  values: T;
};

type ViewResourceProps<Info> = {
  FormComponent?: ComponentType<ViewFormState<Info>>;
};
type ToggleChange = ToolbarProps["onCheckedValueChange"];

function ViewResource<Info>({ FormComponent }: ViewResourceProps<Info>) {
  const styles = useStyles();
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [values, setValues] = useState<Info>({} as Info);
  const [checkedValues, setCheckedValues] = useState<Record<string, string[]>>({
    display: [],
  });
  const onChange: ToggleChange = (_e, { name, checkedItems }) => {
    setCheckedValues((s) => {
      return s ? { ...s, [name]: checkedItems } : { [name]: checkedItems };
    });
  };
  const showJson = useMemo(
    () => checkedValues.display.includes("json") || !FormComponent,
    [checkedValues, FormComponent],
  );

  const onMount: OnMount = useCallback(
    (editor) => {
      editorRef.current = editor;
    },
    [editorRef],
  );

  const { update, scope } = useExplorer();
  const { typeName, schemaName, getFn } = useTreeFunctions(scope ?? []);
  const onCancel = useCallback(() => {
    update({});
  }, [update]);
  const { data, status } = useQuery({
    queryKey: ["get", ...(scope ?? [])],
    queryFn: () => getFn(),
  });

  const onEdit = useCallback(() => {
    update((curr) => {
      return { ...curr, display: "edit" };
    });
  }, [update]);

  useEffect(() => {
    if (status === "success") {
      setValues(data as Info);
    }
  }, [status, data, editorRef.current]);

  useEffect(() => {
    if (editorRef.current && status === "success") {
      editorRef.current.setValue(JSON.stringify(values, null, 2));
    }
  }, [values, status]);

  return (
    <div className={styles.root}>
      <Toolbar
        className={styles.toolbar}
        size="medium"
        checkedValues={checkedValues}
        onCheckedValueChange={onChange}
      >
        <ToolbarButton
          appearance="subtle"
          icon={<ArrowLeftRegular />}
          onClick={onCancel}
        />
        <Text>{`View ${typeName}`}</Text>
        <ToolbarGroup>
          {!!FormComponent && (
            <ToolbarToggleButton
              aria-label="Toggle JSON editor"
              icon={<BracesRegular />}
              name="display"
              value="json"
            />
          )}
          <ToolbarButton
            appearance="subtle"
            icon={<EditRegular />}
            onClick={onEdit}
          >
            Edit
          </ToolbarButton>
        </ToolbarGroup>
      </Toolbar>
      {!showJson && FormComponent && (
        <div className={styles.content}>
          <FormComponent values={values} />
        </div>
      )}
      {
        <div className={showJson ? styles.editor : styles.editorHidden}>
          <JsonEditor onMount={onMount} typeName={schemaName} readOnly={true} />
        </div>
      }
    </div>
  );
}

export default ViewResource;
