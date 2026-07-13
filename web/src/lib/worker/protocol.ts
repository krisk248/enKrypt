// Shared wire types for the crypto worker RPC.

export interface RpcRequest {
  id: number;
  method: string;
  args: unknown[];
}

export interface RpcError {
  code: string;
  message: string;
}

export type RpcResponse =
  | { id: number; ok: true; result: unknown }
  | { id: number; ok: false; error: RpcError };
