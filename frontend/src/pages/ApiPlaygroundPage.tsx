import { ApiPlayground } from '../components/ApiPlayground'

const ApiPlaygroundPage = () => (
  <ApiPlayground baseUrl={import.meta.env.VITE_API_BASE_URL ?? ''} />
)

export default ApiPlaygroundPage
