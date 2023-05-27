# Redirects

A redirect is the server sending back an instruction that further action needs to be taken by the user agent in order to fulfill the request, instead of giving back the contents the client expected.
All redirects also need to send back a `location` header with the new URI to fetch next, which can be absolute or relative.

Redirect in result metrics combines all time spent before the final request starts. All other timers start after this one and only consider the time of the last request in the chain.

Garmata will never automatically follow redirections as per definition a client SHOULD detect and intervene in cyclical redirections. Please set the `max_redirects` value to follow redirects. No breakdown of redirect timings is provided, since redirects by design are expected to be as slim as possible. If you are interested in analysing the redirect times, reduce the `max_redirects` value telling Garmata that you do not want to follow the last redirect to the next target.

The established TCP stream is not reused for the new request. Instead, a new TCP connection is established with the server specified in the redirected URL. This is because the redirect might point to a different server or a different resource on the same server, and therefore a new connection is required to retrieve the correct content.

If you desire you can read up on redirects in [RFC7131](https://datatracker.ietf.org/doc/html/rfc7231#section-6.4) and on [Mozilla](https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections).
