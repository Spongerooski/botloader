import { useState } from "react"

type Props = {
    label: string,
    onClick: () => any,
}

export function AsyncOpButton(props: Props) {
    const [status, setStatus] = useState<boolean>(false);


    async function doOp() {
        setStatus(true);
        await props.onClick();
        setStatus(false);
    }

    return <button disabled={status} onClick={() => doOp()}>{props.label}</button>
}