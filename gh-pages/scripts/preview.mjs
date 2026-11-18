import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const port = Number(process.env.PORT || 4173);
const mime = {
	'.html': 'text/html; charset=utf-8',
	'.css': 'text/css',
	'.js': 'text/javascript',
	'.json': 'application/json',
	'.svg': 'image/svg+xml',
	'.png': 'image/png',
	'.jpg': 'image/jpeg',
	'.webp': 'image/webp',
	'.woff2': 'font/woff2'
};

createServer(async (req, res) => {
	try {
		const url = new URL(req.url, 'http://localhost');
		const pathname = decodeURIComponent(url.pathname);
		if (pathname === '/') {
			res.writeHead(302, { location: '/cross-shelf/' }).end();
			return;
		}
		if (!pathname.startsWith('/cross-shelf/')) throw new Error('Not found');
		let path = resolve(root, `.${pathname.slice('/cross-shelf'.length)}`);
		if (path !== resolve(root) && !path.startsWith(root.endsWith(sep) ? root : root + sep))
			throw new Error('Not found');
		if ((await stat(path)).isDirectory()) path = resolve(path, 'index.html');
		const data = await readFile(path);
		res.writeHead(200, { 'content-type': mime[extname(path)] || 'application/octet-stream' });
		res.end(data);
	} catch {
		res.writeHead(404).end('Not found');
	}
}).listen(port, '127.0.0.1', () => {
	console.log(`CrossShelf landing: http://127.0.0.1:${port}/cross-shelf/`);
});
