export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

export interface Header {
  key: string;
  value: string;
  enabled: boolean;
}

export interface QueryParam {
  key: string;
  value: string;
  enabled: boolean;
}

export interface AuthConfig {
  type: 'none' | 'bearer' | 'api_key' | 'stellar';
  token?: string;
  apiKey?: string;
  stellarPublicKey?: string;
}

export interface ApiRequest {
  id: string;
  name: string;
  method: HttpMethod;
  url: string;
  headers: Header[];
  queryParams: QueryParam[];
  body: string;
  auth: AuthConfig;
}

export interface ApiResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  durationMs: number;
  timestamp: string;
}

export interface HistoryEntry {
  id: string;
  request: ApiRequest;
  response: ApiResponse | null;
  error: string | null;
  timestamp: string;
}

export interface ExampleRequest {
  id: string;
  category: string;
  name: string;
  description: string;
  request: Omit<ApiRequest, 'id' | 'name'>;
}

export const EXAMPLE_REQUESTS: ExampleRequest[] = [
  {
    id: 'ex-1',
    category: 'Contracts',
    name: 'List Wastes',
    description: 'Fetch paginated list of waste records',
    request: {
      method: 'GET',
      url: '/api/contracts/wastes',
      headers: [{ key: 'Accept', value: 'application/json', enabled: true }],
      queryParams: [
        { key: 'page', value: '1', enabled: true },
        { key: 'limit', value: '20', enabled: true },
      ],
      body: '',
      auth: { type: 'none' },
    },
  },
  {
    id: 'ex-2',
    category: 'Contracts',
    name: 'Get Contract Stats',
    description: 'Retrieve overall contract statistics',
    request: {
      method: 'GET',
      url: '/api/contracts/stats',
      headers: [{ key: 'Accept', value: 'application/json', enabled: true }],
      queryParams: [],
      body: '',
      auth: { type: 'none' },
    },
  },
  {
    id: 'ex-3',
    category: 'Analytics',
    name: 'Get Analytics',
    description: 'Fetch analytics metrics for a participant',
    request: {
      method: 'GET',
      url: '/api/analytics',
      headers: [{ key: 'Accept', value: 'application/json', enabled: true }],
      queryParams: [{ key: 'participant_id', value: 'GXXXX...', enabled: true }],
      body: '',
      auth: { type: 'bearer' },
    },
  },
  {
    id: 'ex-4',
    category: 'Export',
    name: 'Export CSV',
    description: 'Export waste data as CSV',
    request: {
      method: 'POST',
      url: '/api/export',
      headers: [
        { key: 'Content-Type', value: 'application/json', enabled: true },
        { key: 'Accept', value: 'application/json', enabled: true },
      ],
      queryParams: [],
      body: JSON.stringify({ format: 'csv', data_type: 'wastes', anonymize: false }, null, 2),
      auth: { type: 'bearer' },
    },
  },
  {
    id: 'ex-5',
    category: 'Audit',
    name: 'Get Audit Logs',
    description: 'Retrieve recent audit log entries',
    request: {
      method: 'GET',
      url: '/api/audit',
      headers: [{ key: 'Accept', value: 'application/json', enabled: true }],
      queryParams: [
        { key: 'page', value: '1', enabled: true },
        { key: 'limit', value: '50', enabled: true },
      ],
      body: '',
      auth: { type: 'bearer' },
    },
  },
  {
    id: 'ex-6',
    category: 'Verification',
    name: 'Verify Record',
    description: 'Submit a waste record for verification',
    request: {
      method: 'POST',
      url: '/api/verification/verify',
      headers: [
        { key: 'Content-Type', value: 'application/json', enabled: true },
        { key: 'X-CSRF-Token', value: '<csrf-token>', enabled: true },
      ],
      queryParams: [],
      body: JSON.stringify({ record_id: 'waste-001', verifier_id: 'GXXXX...' }, null, 2),
      auth: { type: 'bearer' },
    },
  },
];
