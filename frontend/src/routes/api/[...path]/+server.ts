import type { RequestHandler } from '@sveltejs/kit';

const BACKEND = 'https://pmpsti-rag.onrender.com';

async function proxy(request: Request, path: string): Promise<Response> {
  const url = `${BACKEND}/api/${path}`;

  // Forward semua header relevan
  const headers: Record<string, string> = {
    'Content-Type': request.headers.get('Content-Type') ?? 'application/json',
  };

  const headersToForward = ['Authorization', 'X-Guest-Token', 'X-Api-Key', 'Cookie'];
  for (const h of headersToForward) {
    const val = request.headers.get(h);
    if (val) headers[h] = val;
  }

  const body = request.method !== 'GET' && request.method !== 'HEAD'
    ? await request.text()
    : undefined;

  const res = await fetch(url, {
    method: request.method,
    headers,
    body,
  });

  const resBody = await res.text();

  return new Response(resBody, {
    status: res.status,
    headers: {
      'Content-Type': res.headers.get('Content-Type') ?? 'application/json',
    },
  });
}

export const GET: RequestHandler = ({ request, params }) =>
  proxy(request, params.path ?? '');

export const POST: RequestHandler = ({ request, params }) =>
  proxy(request, params.path ?? '');

export const PATCH: RequestHandler = ({ request, params }) =>
  proxy(request, params.path ?? '');

export const DELETE: RequestHandler = ({ request, params }) =>
  proxy(request, params.path ?? '');

export const PUT: RequestHandler = ({ request, params }) =>
  proxy(request, params.path ?? '');
