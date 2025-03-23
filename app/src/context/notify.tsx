import {
  Toast,
  Toaster,
  ToastIntent,
  ToastTitle,
  useToastController,
  useId,
} from "@fluentui/react-components";
import React, { createContext, useCallback, ReactNode } from "react";

export const NotifyContext = createContext<
  (intent: ToastIntent, message: ReactNode) => void
>(() => {});

export const useNotify = () => React.useContext(NotifyContext);

export function NotifyProvider({ children }: { children: React.ReactNode }) {
  const toasterId = useId("toaster");
  const { dispatchToast } = useToastController(toasterId);
  const notify = useCallback(
    (intent: ToastIntent, message: ReactNode) =>
      dispatchToast(
        <Toast>
          <ToastTitle>{message}</ToastTitle>
        </Toast>,
        { position: "bottom-end", intent },
      ),
    [],
  );

  return (
    <NotifyContext.Provider value={notify}>
      <>
        <Toaster toasterId={toasterId} />
        {children}
      </>
    </NotifyContext.Provider>
  );
}
