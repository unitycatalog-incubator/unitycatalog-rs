import "./App.css";
import {
  makeStyles,
  tokens,
  Toolbar,
  ToolbarButton,
  ToolbarRadioButton,
  ToolbarRadioGroup,
  ToolbarProps,
  ToolbarDivider,
} from "@fluentui/react-components";
import Explorer from "./components/Explorer";
import Simulation from "./simulation";
import {
  SettingsRegular,
  AlignCenterHorizontal24Regular,
  AlignLeft24Regular,
  AlignRight24Regular,
} from "@fluentui/react-icons";
import { useState } from "react";

const useStyles = makeStyles({
  root: {
    display: "flex",
    height: "100vh",
    width: "100vw",
    flexDirection: "column",
  },

  toolbar: {
    borderBottomColor: tokens.colorNeutralForeground4,
    borderBottomWidth: "1px",
    borderBottomStyle: "solid",
  },

  content: {
    flex: 1,
  },
});

function App() {
  const styles = useStyles();

  const [checkedValues, setCheckedValues] = useState<Record<string, string[]>>({
    rootView: ["catalogs"],
  });

  const onChange: ToolbarProps["onCheckedValueChange"] = (
    _e,
    { name, checkedItems },
  ) => {
    setCheckedValues((s) => {
      return s ? { ...s, [name]: checkedItems } : { [name]: checkedItems };
    });
  };

  return (
    <div className={styles.root}>
      <div className={styles.toolbar}>
        <Toolbar
          size="medium"
          checkedValues={checkedValues}
          onCheckedValueChange={onChange}
        >
          <ToolbarButton appearance="subtle" icon={<SettingsRegular />} />
          <ToolbarDivider />
          <ToolbarRadioGroup>
            <ToolbarRadioButton
              aria-label="Catalogs View"
              name="rootView"
              value="catalogs"
              icon={<AlignLeft24Regular />}
            />
            <ToolbarRadioButton
              aria-label="Simulation View"
              name="rootView"
              value="simulation"
              icon={<AlignCenterHorizontal24Regular />}
            />
            <ToolbarRadioButton
              aria-label="Hydrofoil View"
              name="rootView"
              value="hydrofoil"
              icon={<AlignRight24Regular />}
            />
          </ToolbarRadioGroup>
        </Toolbar>
      </div>
      <div className={styles.content}>
        {checkedValues.rootView.includes("catalogs") && <Explorer />}
        {checkedValues.rootView.includes("simulation") && <Simulation />}
      </div>
    </div>
  );
}

export default App;
