# srvrls - a lightweight serverless framework

This library is likely not for you. It has opinions, strong ones, likely not your own.

We built it to simplify building applications rapidly within a serverless architecture.
It also serves to add some consistency around serverless applications that we otherwise
did not have.

Our design priorities here are simple:

- reduce boilerplate to building serverless function applications
- provide opinionated defaults to otherwise open questions (more on this later)
- provide decoupling between the serverless function provider and the application logic

## Opinions

- the application will be accessed via HTTP - standard HTTP concepts such as headers, query 
and path parameters are closely held
- differentiation between `None` and `Some(String)` provides minimal value if you're just matching 
on the string value - we remove `Option`s where they are unnecessary to again reduce boilerplate
- every Rust dev has their own preferred way of solving a problem - as much as we enjoy our own
opinions, we realize that taking that too far makes our solution unpalatable to many.
Accordingly we keep it simple, reduce to boilerplate and leave the pattern to the dev.
- most of us are probably using AWS Lambda - we built it that way. Expect changes whenever
we get around to building out the Google Cloud and Azure implementations (don't wait up).
- TBH, we really targeted AWS Gateway Proxy - so if you're doing  anything other than that,
your application probably won't work well with our implementation

