# Fastly CDN setup

Used for all static assets.

Dynamic backends are typically through cloud run and so cannot be efficiently cached right now, at least until Google supports straight external IP or a load balancer.

Make sure to set interconnect location that's closest to storage (e.g. Amsterdam Fastly for Belgium Google)

Only each origin should have host request condition, in order for that origin to be used for the domain. e.g. `req.http.host == "docs.jigzi.org"`

A small `VCL Snippet` for the `recv` block is required to make it fetch index.html for plain directory requests:

```
if (req.url ~ "\/$") {
  set req.url = req.url "index.html";
}
```

Though we're not using that at the moment (pushing to firebase for docs and storybook).

We are using this VCL in order to cache things for 1 year, and then we purge things more aggressively as needed:

```
set beresp.ttl = 1y;
if (beresp.status >= 200 && beresp.status < 300) {
  set beresp.cacheable = true;
}
```

See Fastly documentation for more details

# Purging

Some buckets are purged automatically via a google cloud function (see [google cloud](../google_cloud/google_cloud.md)) on every file change

Others are not and require manual purging, either because they are never expected to be purged (i.e. uploads) or it would depend on the exact route and very rarely need to be purged (i.e. page templates, if/when that is supported for CDN cache)

For buckets that are purged automatically, the file's cache-control headers are set to not cache, as per Fastly's recommendation. This setting is done immediately before the file is purged

Note that purging only happens when files are changed or added, not when deleted (handling deleting and/or archiving would require an additional cloud function for each origin, and missing files aren't a common use case for the end user. a manual purge, of course, can always be done if there's a real need to remove its existence from the edge cache)