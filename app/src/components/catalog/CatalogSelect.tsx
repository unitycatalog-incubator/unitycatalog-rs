import { useState, useEffect } from "react";
import { TreeProvider } from "../../context";
import {
  Button,
  FlatTree,
  FlatTreeProps,
  TreeItemValue,
  DrawerBody,
  DrawerHeader,
  DrawerHeaderTitle,
  OverlayDrawer,
  useRestoreFocusSource,
  useRestoreFocusTarget,
  DrawerFooter,
} from "@fluentui/react-components";
import { AddRegular } from "@fluentui/react-icons";
import CatalogTree from "../catalog/CatalogTree";
import { Dismiss24Regular } from "@fluentui/react-icons";

type OnTreeSelect = FlatTreeProps["onCheckedChange"];

export type SelectedItem = {
  kind: "catalog" | "schema" | "table";
  fullName: string;
};

type CatalogSelectProps = {
  selectionMode: "single" | "multiselect";
  onAdd?: (selected: SelectedItem[]) => void;
};

function CatalogSelect({ onAdd, selectionMode }: CatalogSelectProps) {
  const restoreFocusTargetAttributes = useRestoreFocusTarget();
  const restoreFocusSourceAttributes = useRestoreFocusSource();
  const [isOpen, setIsOpen] = useState(false);
  const [checked, setChecked] = useState<TreeItemValue[]>([]);
  const [items, setItems] = useState<SelectedItem[]>([]);

  const onChange: OnTreeSelect = (_ev, data) => {
    setChecked([data.value]);
  };

  useEffect(() => {
    if (checked.length > 0) {
      const item = checked[0].toString().split(".").slice(1);
      const kind =
        item.length === 1 ? "catalog" : item.length === 2 ? "schema" : "table";
      const fullName = item.join(".");

      setItems([{ kind, fullName }]);
    }
  }, [checked]);

  return (
    <div>
      <OverlayDrawer
        as="aside"
        {...restoreFocusSourceAttributes}
        open={isOpen}
        onOpenChange={(_, { open }) => setIsOpen(open)}
        position="end"
      >
        <DrawerHeader>
          <DrawerHeaderTitle
            action={
              <Button
                appearance="subtle"
                aria-label="Close"
                icon={<Dismiss24Regular />}
                onClick={() => setIsOpen(false)}
              />
            }
          >
            Select Object
          </DrawerHeaderTitle>
        </DrawerHeader>

        <DrawerBody>
          <FlatTree
            selectionMode={selectionMode}
            checkedItems={checked}
            onCheckedChange={onChange}
          >
            <TreeProvider value={["catalogs"]}>
              <CatalogTree setSize={1} setPos={1} />
            </TreeProvider>
          </FlatTree>
        </DrawerBody>
        <DrawerFooter>
          <Button
            appearance="primary"
            onClick={() => {
              onAdd?.(items);
              setIsOpen(false);
              setChecked([]);
            }}
          >
            Select
          </Button>
          <Button appearance="secondary" onClick={() => setIsOpen(false)}>
            Cancel
          </Button>
        </DrawerFooter>
      </OverlayDrawer>

      <Button
        {...restoreFocusTargetAttributes}
        appearance="subtle"
        onClick={() => setIsOpen(true)}
        icon={<AddRegular />}
      >
        Add
      </Button>
    </div>
  );
}

export default CatalogSelect;
