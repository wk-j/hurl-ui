# Get comments for a post
GET https://jsonplaceholder.typicode.com/posts/1/comments
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$" count == 5
jsonpath "$[0].postId" == 1
jsonpath "$[0].email" exists
jsonpath "$[0].body" exists
