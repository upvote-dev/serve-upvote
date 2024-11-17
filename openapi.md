# serve-upvote

> Version 0.0.1

Upvote endpoints

## Path Table

| Method | Path | Description |
| --- | --- | --- |
| GET | [/api](#getapi) | Versions of this package and its first-party dependencies |
| POST | [/api/token](#postapitoken) | Generate a token for a grant flow |
| GET | [/api/v0/profile](#getapiv0profile) | Get profile |
| POST | [/api/v0/profile](#postapiv0profile) | Upsert Profile |
| POST | [/api/v0/review](#postapiv0review) | Upsert Review |
| GET | [/api/v0/review/{id}](#getapiv0reviewid) | Get Review by id |
| GET | [/api/v0_noauth/reviews](#getapiv0_noauthreviews) | Get Reviews |
| POST | [/secured/logout](#postsecuredlogout) | Logout a user (uses provided Bearer token from Header) |
| GET | [/secured/secret](#getsecuredsecret) | Shows secret to authenticated user (uses provided Bearer token from Header) |

## Reference Table

| Name | Path | Description |
| --- | --- | --- |
| NewProfileJ | [#/components/schemas/NewProfileJ](#componentsschemasnewprofilej) | Create a new Profile with this record |
| NewReviewJ | [#/components/schemas/NewReviewJ](#componentsschemasnewreviewj) | Create a new Review with this record |
| TokenRequest | [#/components/schemas/TokenRequest](#componentsschemastokenrequest) |  |
| password | [#/components/securitySchemes/password](#componentssecurityschemespassword) |  |

## Path Details

***

### [GET]/api

- Summary  
Versions of this package and its first-party dependencies

#### Responses

***

### [POST]/api/token

- Summary  
Generate a token for a grant flow

#### RequestBody

- application/x-www-form-urlencoded

```ts
{
  // optional client ID (as used, for example, in RFC6749's non password non refresh grant flow)
  client_id?: string | null
  // optional client secret (as used, e.g., in RFC6749's non (password|refresh) grant flow)
  client_secret?: string | null
  // RFC6749 grant type
  grant_type: string
  // optional password (as used, for example, in RFC6749's password grant flow)
  password?: string | null
  // optional username (as used, for example, in RFC6749's password grant flow)
  username?: string | null
}
```

#### Responses

- 200 Token created

- 400 Unauthorized User

- 404 Not Found User

- 500 Bad Request

***

### [GET]/api/v0/profile

- Summary  
Get profile

- Security  
password  

#### Responses

- 200 Profile for user associated with access token

- 404 Not found: User does not have associated profile

***

### [POST]/api/v0/profile

- Summary  
Upsert Profile

- Security  
password  

#### RequestBody

- application/json

```ts
// Create a new Profile with this record
{
  // Optional alias (if set, this is publicly used instead of username)
  alias?: string | null
  // Optional starting coin amount
  coins?: integer | null
  // Optional image URL to avatar associated with this profile
  profile_image_url?: string | null
  // Optional rank (TBD rank ontology)
  rank?: string | null
  // Optional username (regardless of whether set uses username from access token)
  username?: string
}
```

#### Responses

- 200 Profile created

- 401 Unauthorised: You tried to create a profile for another user

***

### [POST]/api/v0/review

- Summary  
Upsert Review

- Security  
password  

#### RequestBody

- application/json

```ts
// Create a new Review with this record
{
  // Optional free-text review
  message?: string | null
  // Optional image URL to photo of reviewee
  photo_url?: string | null
  // Unique identifier to object being reviewed
  reviewee: string
  // Type of object being reviewed
  reviewee_kind: string
  // Optional username (regardless of whether set uses username from access token)
  username?: string | null
  // Optional video URL to video of reviewee
  video_url?: string | null
  // Appraisal (e.g., `-1` is downvote, `1` is upvote)
  vote: integer
}
```

#### Responses

- 200 Review created

- 500 Internal Server Error

***

### [GET]/api/v0/review/{id}

- Summary  
Get Review by id

#### Responses

- 200 Review found from database

- 404 Not found

***

### [GET]/api/v0_noauth/reviews

- Summary  
Get Reviews

#### Responses

- 200 Reviews found in database

- 404 Not found

***

### [POST]/secured/logout

- Summary  
Logout a user (uses provided Bearer token from Header)

- Security  
password  

#### Responses

- 200 

***

### [GET]/secured/secret

- Summary  
Shows secret to authenticated user (uses provided Bearer token from Header)

- Security  
password  

#### Responses

- 200 secret endpoint

## References

### #/components/schemas/NewProfileJ

```ts
// Create a new Profile with this record
{
  // Optional alias (if set, this is publicly used instead of username)
  alias?: string | null
  // Optional starting coin amount
  coins?: integer | null
  // Optional image URL to avatar associated with this profile
  profile_image_url?: string | null
  // Optional rank (TBD rank ontology)
  rank?: string | null
  // Optional username (regardless of whether set uses username from access token)
  username?: string
}
```

### #/components/schemas/NewReviewJ

```ts
// Create a new Review with this record
{
  // Optional free-text review
  message?: string | null
  // Optional image URL to photo of reviewee
  photo_url?: string | null
  // Unique identifier to object being reviewed
  reviewee: string
  // Type of object being reviewed
  reviewee_kind: string
  // Optional username (regardless of whether set uses username from access token)
  username?: string | null
  // Optional video URL to video of reviewee
  video_url?: string | null
  // Appraisal (e.g., `-1` is downvote, `1` is upvote)
  vote: integer
}
```

### #/components/schemas/TokenRequest

```ts
{
  // optional client ID (as used, for example, in RFC6749's non password non refresh grant flow)
  client_id?: string | null
  // optional client secret (as used, e.g., in RFC6749's non (password|refresh) grant flow)
  client_secret?: string | null
  // RFC6749 grant type
  grant_type: string
  // optional password (as used, for example, in RFC6749's password grant flow)
  password?: string | null
  // optional username (as used, for example, in RFC6749's password grant flow)
  username?: string | null
}
```

### #/components/securitySchemes/password

```ts
{
  "type": "oauth2",
  "flows": {
    "password": {
      "tokenUrl": "/api/token",
      "scopes": {}
    }
  }
}
```