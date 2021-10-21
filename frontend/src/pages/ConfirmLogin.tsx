import { useEffect, useState } from "react"
import { Redirect, useLocation } from "react-router";
import { ApiClient, isErrorResponse, ApiError } from "../ApiClient";

interface LoadingStatus {
    loading: boolean,
    error?: ApiError,
}

export function ConfirmLoginPage() {
    let [status, setStatus] = useState<LoadingStatus>({
        loading: true,
    });

    let loc = useLocation();

    useEffect(() => {
        let query = new URLSearchParams(loc.search);

        async function completeLogin() {
            let client = new ApiClient();
            let resp = await client.confirmLogin(query.get("code") as string, query.get("state") as string)
            if (!isErrorResponse(resp)) {
                localStorage.setItem("botloader_token", resp.token);
                setStatus({
                    loading: false,
                });
            } else {
                setStatus({
                    loading: false,
                    error: resp,
                });
            }


        }

        completeLogin()

    }, [loc])

    if (status.loading) {
        return <h2>Logging you in...</h2>
    } else {
        if (status.error) {
            return <p>Failed logging you in: <code>{JSON.stringify(status.error)}</code></p>
        } else {
            return <Redirect to="/servers"></Redirect>
        }
    }
}