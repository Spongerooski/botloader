export interface Attachment {
    contentType: string | null;
    filename: string;
    height: bigint | null;
    id: string;
    proxyUrl: string;
    size: bigint;
    url: string;
    width: bigint | null;
}
