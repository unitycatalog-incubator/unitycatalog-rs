import { useCallback, useState, useEffect } from "react";
import ucClient, { ShareInfo, UpdateShareRequest } from "../../client";
import { InputChange } from "../../types";
import { CreateFormState, Input, CreateResource } from "../forms";
import { useQuery } from "@tanstack/react-query";
import { useTreeContext } from "../../context";

function UpdateShareForm({
  values,
  setValues,
}: CreateFormState<UpdateShareRequest>) {
  const onNameChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, name: data.value }));
  }, []);
  const onCommentChange: InputChange = useCallback((_ev, data) => {
    setValues((curr) => ({ ...curr, comment: data.value }));
  }, []);

  return (
    <>
      <Input label="Name" value={values.name ?? ""} onChange={onNameChange} />
      <Input
        label="Comment"
        onChange={onCommentChange}
        value={values.comment ?? ""}
      />
    </>
  );
}

function UpdateShare() {
  const [values, setValues] = useState<ShareInfo>({});
  const queryKey = useTreeContext();

  const { data, status } = useQuery({
    queryKey: ["get", ...queryKey],
    queryFn: ({ queryKey }) =>
      ucClient.shares.get({ name: queryKey[queryKey.length - 1] }),
  });

  useEffect(() => {
    if (status === "success") {
      setValues(data);
    }
  }, [status, data]);

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
