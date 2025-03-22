import {
    Button,
    Dialog,
    DialogActions,
    DialogBody,
    DialogContent,
    DialogSurface,
    DialogTrigger,
    makeStyles,
    tokens,
    Text,
    Tag,
} from "@fluentui/react-components";
import { Delete20Regular } from "@fluentui/react-icons";
import { useState, useCallback, useMemo } from "react";
import { useTreeContext, useNotify, useTreeFunctions } from "../context";
import { useMutation, useQueryClient } from "@tanstack/react-query";

const useStyles = makeStyles({
    delete: {
        backgroundColor: tokens.colorStatusDangerBackground3,
        "&:hover": {
            backgroundColor: tokens.colorStatusDangerBackground3Hover,
        },
    },

    deleteIcon: {
        "&:hover": {
            color: tokens.colorStatusDangerBackground3Hover,
        },
    },
});

function DeleteDialogInner() {
    const [open, setOpen] = useState(false);
    const styles = useStyles();

    const scope = useTreeContext();
    const { typeName, deleteFn } = useTreeFunctions(scope);

    const nameTag = useMemo(() => {
        const fullName = scope.slice(1).join(".");
        return <Tag>{fullName}</Tag>;
    }, [scope]);

    const notify = useNotify();
    const queryClient = useQueryClient();
    const mutation = useMutation({
        mutationFn: deleteFn,
        onError: () => {
            const message = (
                <span>
                    <Text>{`Failed to delete ${typeName.toLowerCase()}`}</Text>
                    {nameTag}
                </span>
            );
            notify("error", message);
        },
        onSuccess: () => {
            setOpen(false);
            const message = (
                <span>
                    <Text>{typeName}</Text>
                    {nameTag}
                    <Text>deleted successfully</Text>
                </span>
            );
            notify("success", message);
            // invalidate list query in parent scope
            queryClient.invalidateQueries({
                queryKey: scope.slice(0, scope.length - 1),
            });
        },
    });
    const onClick = useCallback(() => mutation.mutate(), [mutation]);

    return (
        <Dialog open={open} onOpenChange={(_ev, data) => setOpen(data.open)}>
            <DialogTrigger disableButtonEnhancement>
                <Button
                    icon={<Delete20Regular className={styles.deleteIcon} />}
                    appearance="subtle"
                    title="Delete"
                />
            </DialogTrigger>
            <DialogSurface>
                <DialogBody>
                    <DialogContent>
                        <Text>{`Are you sure you want to delete ${typeName}`}</Text>
                        {nameTag}
                    </DialogContent>
                    <DialogActions>
                        <Button
                            className={styles.delete}
                            appearance="primary"
                            onClick={onClick}
                        >
                            Delete
                        </Button>
                        <DialogTrigger disableButtonEnhancement>
                            <Button appearance="secondary">Cancel</Button>
                        </DialogTrigger>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    );
}

function DeleteDialog() {
    const scope = useTreeContext();
    if (scope.length > 1) {
        return <DeleteDialogInner />;
    }
    return null;
}

export default DeleteDialog;
