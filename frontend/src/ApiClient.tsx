import { BuildConfig } from "./BuildConfig";

export class ApiClient {
    token?: string;

    constructor(token?: string) {
        this.token = token;
    }

    async do<T>(method: string, path: string, body?: any): Promise<ApiResult<T>> {
        let base = BuildConfig.botloaderApiBase;

        let headers = {};
        if (this.token) {
            headers = {
                // eslint-disable-next-line @typescript-eslint/naming-convention
                Authorization: this.token,
                ...headers,
            };
        }

        if (body) {
            headers = {
                // eslint-disable-next-line @typescript-eslint/naming-convention
                "Content-Type": "application/json",
                ...headers,
            };
        }

        let response = await fetch(base + path, {
            headers: headers,
            method: method,
            body: body ? JSON.stringify(body) : undefined,
        });

        if (response.status !== 200) {
            let decoded: ApiErrorResponse = await response.json();
            return {
                resp_code: response.status,
                is_error: true,
                response: decoded,
            };
        }

        return await response.json();
    }

    async get<T>(path: string,): Promise<ApiResult<T>> {
        return await this.do("GET", path);
    }

    async post<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("POST", path, body);
    }

    async delete<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("DELETE", path, body);
    }

    async put<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("PUT", path, body);
    }

    async patch<T>(path: string, body?: any): Promise<ApiResult<T>> {
        return await this.do("PATCH", path, body);
    }

    async getCurrentUser(): Promise<ApiResult<User>> {
        return await this.get("/api/current_user");
    }

    async getCurrentUserGuilds(): Promise<ApiResult<CurrentGuildsResponse>> {
        return await this.get("/api/guilds");
    }

    async getAllSessions(): Promise<ApiResult<SessionMeta[]>> {
        return await this.get("/api/sessions");
    }

    async logout(): Promise<ApiResult<{}>> {
        return await this.post("/api/logout");
    }

    async deleteSession(token: string): Promise<ApiResult<{}>> {
        return await this.delete("/api/sessions", {
            token: token,
        });
    }

    async deleteAllSessions(): Promise<ApiResult<{}>> {
        return await this.delete("/api/sessions/all");
    }

    async createApiToken(): Promise<ApiResult<SessionMeta>> {
        return await this.put("/api/sessions");
    }

    async confirmLogin(code: string, state: string): Promise<ApiResult<LoginResponse>> {
        return await this.post("/api/confirm_login", {
            code: code,
            state: state,
        });
    }

    async getAllScripts(guildId: string,): Promise<ApiResult<Script[]>> {
        return await this.get(`/api/guilds/${guildId}/scripts`);
    }

    async createScript(guildId: string, data: CreateScript): Promise<ApiResult<Script>> {
        return await this.put(`/api/guilds/${guildId}/scripts`, data);
    }

    async updateScript(guildId: string, id: number, data: UpdateScript): Promise<ApiResult<Script>> {
        return await this.patch(`/api/guilds/${guildId}/scripts/${id}`, data);
    }

    async delScript(guildId: string, id: number): Promise<ApiResult<{}>> {
        return await this.delete(`/api/guilds/${guildId}/scripts/${id}`);
    }
}

type ApiResult<T> = T | ApiError;

export function isErrorResponse<T>(resp: ApiResult<T>): resp is ApiError {
    return (resp as ApiError).is_error !== undefined;
}

export interface ApiError {
    resp_code: number,
    is_error: true,
    response?: ApiErrorResponse,
}

export interface ApiErrorResponse {
    code: number,
    description: string,
}

export function ApiClientInjector() { }

export interface User {
    avatar?: string,
    bot: boolean,
    discriminator: string,
    email?: string,
    flags?: number,
    id: string,
    locale?: string,
    username: string,
    premium_type?: number,
    public_flags?: number,
    verified?: boolean,
}


export interface UserGuild {
    id: string,
    name: string,
    icon?: string,
    owner: boolean,
    permissions: string,
    features: string[],
}

export interface BotGuild {
    guild: UserGuild,
    connected: boolean,
}

export interface CurrentGuildsResponse {
    guilds: BotGuild[],
}

export interface LoginResponse {
    user: User,
    token: string,
}

export interface SessionMeta {
    kind: SessionType,
    created_at: string,
    token: string,
}

export type SessionType = "User" | "ApiKey";


export interface Script {
    id: number,
    name: string,
    original_source: string,
    compiled_js: string,
    enabled: boolean,
}

export interface CreateScript {
    name: string,
    original_source: string,
    enabled: boolean,
}

export interface UpdateScript {
    name: string,
    original_source: string,
    enabled: boolean,
}