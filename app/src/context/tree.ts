import { createContext, useContext, useMemo } from "react";
import ucClient from "../client";

const TreeContext = createContext<string[]>([]);
export const TreeProvider = TreeContext.Provider;
export const useTreeContext = () => useContext(TreeContext);

export const useTreeScope = () => {
    const scope = useTreeContext();
    const parentScope = useMemo(
        () => scope.slice(0, scope.length - 1),
        [scope],
    );
    const parentValue = useMemo(
        () => (parentScope.length > 0 ? parentScope.join(".") : undefined),
        [scope],
    );
    const value = useMemo(() => scope.join("."), [scope]);
    const hasChildren = useMemo(() => {
        if (scope.length === 1) return true;
        return scope[0] === "catalogs" && scope.length <= 3;
    }, [scope]);
    return { scope, value, parentScope, parentValue, hasChildren };
};

export const useTypeName = (scope: string[]) => {
    if (scope.length === 2) {
        switch (scope[0]) {
            case "catalogs":
                return "Catalog";
            case "external_locations":
                return "External location";
            case "shares":
                return "Share";
            case "credentials":
                return "Credential";
            case "recipients":
                return "Recipient";
        }
    }
    if (scope[0] === "catalogs") {
        if (scope.length === 3) return "Schema";
        if (scope.length === 4) return "Table";
    }
    throw new Error(`Unknown scope: ${scope}`);
};

export const useTreeFunctions = (scope: string[]) => {
    const typeName = useTypeName(scope);
    const { deleteFn } = useMemo(() => {
        if (scope.length === 2) {
            switch (scope[0]) {
                case "catalogs":
                    return {
                        deleteFn: () => ucClient.catalogs.delete(scope[1]),
                    };
                case "external_locations":
                    return {
                        deleteFn: () =>
                            ucClient.externalLocations.delete(scope[1]),
                    };
                case "shares":
                    return {
                        deleteFn: () => ucClient.shares.delete(scope[1]),
                    };
                case "credentials":
                    return {
                        deleteFn: () => ucClient.credentials.delete(scope[1]),
                    };
                case "recipients":
                    return {
                        deleteFn: () => ucClient.recipients.delete(scope[1]),
                    };
            }
        }

        if (scope[0] === "catalogs") {
            if (scope.length === 3) {
                return {
                    deleteFn: () =>
                        ucClient.schemas.delete({
                            catalog: scope[1],
                            name: scope[2],
                        }),
                };
            }
            if (scope.length === 4) {
                return {
                    deleteFn: () =>
                        ucClient.tables.delete({
                            catalog: scope[1],
                            schema: scope[2],
                            name: scope[3],
                        }),
                };
            }
        }

        throw new Error(`Unknown scope: ${scope}`);
    }, [scope]);

    return { typeName, deleteFn };
};
