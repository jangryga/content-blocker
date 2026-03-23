from mitmproxy import http

blocked_domains = [
    "reddit.com",
    "discord.com"
]

def request(flow: http.HTTPFlow):
    for domain in blocked_domains:
        if domain in flow.request.pretty_host:
            flow.response = http.Response.make(302, b"", {"Location": "https://skillstech.app/"})