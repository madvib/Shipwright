export const prerender = false;
import type { APIRoute } from 'astro';

export const POST: APIRoute = async ({ request, locals }) => {
    try {
        const { email } = await request.json();

        if (!email || !email.includes('@')) {
            return new Response(JSON.stringify({ error: 'Invalid email' }), { status: 400 });
        }

        // Access Cloudflare KV binding 
        // locals.runtime.env.WAITLIST should be available in Cloudflare env
        const env = (locals as any).runtime?.env;

        if (env?.WAITLIST) {
            const timestamp = new Date().toISOString();
            await env.WAITLIST.put(`waitlist:${email}`, JSON.stringify({ email, timestamp }));
            return new Response(JSON.stringify({ success: true }), { status: 200 });
        } else {
            // For local development or if KV is not configured, we'll log it
            console.log('KV WAITLIST not found. Logging to console:', email);
            return new Response(JSON.stringify({ success: true, local: true }), { status: 200 });
        }

    } catch (error) {
        return new Response(JSON.stringify({ error: 'Server error' }), { status: 500 });
    }
};
