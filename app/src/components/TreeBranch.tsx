import {
    FlatTreeItem,
    TreeItemLayout,
    Spinner,
    Button,
    TreeItemLayoutProps,
} from "@fluentui/react-components";
import { AddRegular } from "@fluentui/react-icons";
import { useQuery } from "@tanstack/react-query";
import {
    ComponentType,
    RefObject,
    useCallback,
    useEffect,
    useRef,
    useState,
} from "react";
import {
    TreeProvider,
    useExplorer,
    useTreeContext,
    useTreeScope,
} from "../context";
import { TreeItemOnChange } from "../types";
import DeleteDialog from "./DeleteDialog";

type Icon = TreeItemLayoutProps["iconBefore"];

type TreeRootProps<Item> = {
    setSize: number;
    setPos: number;
    listFn: () => Promise<Item[]>;
    ItemComponent: ComponentType<{
        info: Item & { name: string };
        ref: RefObject<HTMLDivElement> | null;
        setSize: number;
        setPos: number;
    }>;
    icon: Icon;
    rootName: string;
};

const CreateItem = () => {
    const scope = useTreeContext();
    const { update } = useExplorer();
    const onClick = useCallback(() => {
        update({ display: "create", scope });
    }, [update]);

    return (
        <Button appearance="subtle" onClick={onClick} icon={<AddRegular />} />
    );
};

function TreeBranch<Item extends { name?: string }>({
    setSize,
    setPos,
    listFn,
    icon,
    ItemComponent,
    rootName,
}: TreeRootProps<Item>) {
    const [open, setOpen] = useState(false);
    const onOpenChange: TreeItemOnChange = useCallback(
        (_ev, data) => setOpen(data.open),
        [setOpen],
    );

    const { parentValue, value, scope, hasChildren } = useTreeScope();
    const { data, status } = useQuery({
        queryKey: scope,
        queryFn: listFn,
        enabled: open,
        refetchInterval: 30000,
    });

    const firstItemRef = useRef<HTMLDivElement>(null);
    useEffect(() => {
        if (open && status === "success") firstItemRef.current?.focus();
    }, [open, status]);

    return (
        <>
            <FlatTreeItem
                parentValue={parentValue}
                value={value}
                aria-level={scope.length}
                aria-setsize={setSize}
                aria-posinset={setPos}
                itemType={hasChildren ? "branch" : "leaf"}
                open={open}
                onOpenChange={onOpenChange}
            >
                <TreeItemLayout
                    iconBefore={icon}
                    expandIcon={
                        open && status === "pending" ? (
                            <Spinner size="extra-tiny" />
                        ) : undefined
                    }
                    actions={[<DeleteDialog />, <CreateItem />]}
                >
                    {rootName}
                </TreeItemLayout>
            </FlatTreeItem>
            {open &&
                status === "success" &&
                data.map(
                    (item, index) =>
                        item.name && (
                            <TreeProvider value={[...scope, item.name]}>
                                <ItemComponent
                                    key={`${value}.${item.name}`}
                                    ref={index === 0 ? firstItemRef : null}
                                    // @ts-expect-error
                                    info={item}
                                    setSize={data.length}
                                    setPos={index + 1}
                                />
                            </TreeProvider>
                        ),
                )}
        </>
    );
}

export default TreeBranch;
