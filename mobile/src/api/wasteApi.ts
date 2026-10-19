import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
});

export interface WasteSubmission {
  waste_type: string;
  weight: number;
  submitter: string;
}

export const submitWaste = async (data: WasteSubmission) => {
  const response = await apiClient.post('/api/waste/submit', data);
  return response.data;
};

export const getWaste = async (wasteId: string) => {
  const response = await apiClient.get(`/api/waste/${wasteId}`);
  return response.data;
};

export const transferWaste = async (wasteId: string, to: string) => {
  const response = await apiClient.post(`/api/waste/${wasteId}/transfer`, { to });
  return response.data;
};

export const getParticipantStats = async (address: string) => {
  const response = await apiClient.get(`/api/participants/${address}/stats`);
  return response.data;
};
