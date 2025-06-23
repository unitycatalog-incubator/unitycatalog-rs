import { http, HttpResponse } from 'msw'

import catalogResponse from './responses/catalogs.json'

export const handlers = [
  http.get('https://unitycatalog.io/api/2.1/unity-catalog/catalogs', () => {
    return HttpResponse.json(catalogResponse)
  }),
]
