# ci: build and validate kits with the v3 frontend

Follow-up commit to the v3 migration (blocked from pushing directly by
the token's missing `workflow` scope — apply this file separately or
merge it into the branch with a scoped token).

## .github/workflows/validate.yml

Replace the `sbx kit validate` job (it has no v3 source-kit path) with a
real buildx build of every kit and mixin descriptor; template Dockerfile
`--check` stays:

```yaml
name: Validate

on:
  pull_request:
  push:
    branches: [ main ]
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  # Image contexts from images.json (same source of truth as
  # publish-image.yml) — feeds both jobs' matrices.
  images:
    runs-on: ubuntu-latest
    outputs:
      images: ${{ steps.images.outputs.images }}
    steps:
      - name: Checkout
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1

      - name: Read image list
        id: images
        run: echo "images=$(jq -c '.images' images.json)" >> "$GITHUB_OUTPUT"

  # Every kit and mixin builds with the v3 sandbox-kit frontend (the
  # descriptor's # syntax= line dispatches it). The build validates the
  # descriptor strictly — sbx kit validate has no v3 source-kit path, and
  # buildx --check cannot run foreign frontends, so a real build is the
  # only local validation. No Docker login needed: layers stay local.
  # Pushing happens in publish-image.yml at release time.
  kits-mixins:
    name: Kit or mixin (v3 frontend build)
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include: ${{ fromJson(needs.images.outputs.images) }}
    steps:
      - name: Checkout
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@37fe631027851001ddb9b187196cc803df7f5f0e # v4.3.0

      - name: Build ${{ matrix.context }} (descriptor)
        if: ${{ !startsWith(matrix.context, 'template-') }}
        run: |
          docker buildx build "${{ matrix.context }}" \
            --file "${{ matrix.context }}/${{ matrix.name }}.yaml" \
            --platform linux/amd64

      - name: Build ${{ matrix.context }} (Dockerfile)
        if: ${{ startsWith(matrix.context, 'template-') }}
        run: |
          docker buildx build "${{ matrix.context }}" \
            --platform linux/amd64

  # Static build checks for every publishable template Dockerfile.
  # The --check mode runs the BuildKit frontend checks without executing
  # the build steps; full builds stay in publish-image.yml at release
  # time. (Kit/mixin validation is the kits-mixins job above.)
  dockerfiles:
    name: Dockerfile (buildx check)
    runs-on: ubuntu-latest
    needs: images
    strategy:
      fail-fast: false
      matrix:
        include: ${{ fromJson(needs.images.outputs.images) }}
    steps:
      - name: Checkout
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@37fe631027851001ddb9b187196cc803df7f5f0e # v4.3.0

      - name: Check ${{ matrix.context }}/Dockerfile
        if: ${{ startsWith(matrix.context, 'template-') }}
        run: docker buildx build --check "${{ matrix.context }}"
```

## .github/workflows/publish-image.yml

Build kit and mixin images from their v3 descriptors (two step variants,
keyed on `template-` contexts) alongside the template images:

```yaml
name: Publish images to Docker Hub

on:
  release:
    types: [ published ]

permissions:
  contents: read

env:
  IMAGE_PREFIX: docker.io/nikcio

jobs:
  matrix:
    runs-on: ubuntu-latest
    outputs:
      images: ${{ steps.images.outputs.images }}
    steps:
      - name: Checkout
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1

      - name: Read image list
        id: images
        run: echo "images=$(jq -c '.images' images.json)" >> "$GITHUB_OUTPUT"

  build:
    needs: matrix
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include: ${{ fromJson(needs.matrix.outputs.images) }}
    steps:
      - name: Checkout
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1

      - name: Set VERSION variable from tag
        run: echo "VERSION=${GITHUB_REF/refs\/tags\/v/}" >> $GITHUB_ENV

      - name: Verify published workload kit references this image
        if: ${{ startsWith(matrix.context, 'template-') }}
        run: |
          image="${IMAGE_PREFIX}/${{ matrix.name }}:v${VERSION}"
          grep -qF "$image" "kit-${{ matrix.context }}.kit-${{ matrix.context }}.yaml" || {
            echo "::error::kit-${{ matrix.context }}.kit-${{ matrix.context }}.yaml does not reference $image"
            exit 1
          }

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@37fe631027851001ddb9b187196cc803df7f5f0e # v4.3.0

      - name: Login to Docker Hub
        uses: docker/login-action@dbcb813823bdd20940b903addbd779551569679f # v4.6.0
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      # v3 kit and mixin images build from the descriptor with the
      # sandbox-kit frontend; template images build the plain Dockerfile.
      # Both are ordinary OCI images, tagged with the release version.
      - name: Build and push (v3 kit descriptor)
        if: ${{ !startsWith(matrix.context, 'template-') }}
        uses: docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a # v7.3.0
        with:
          context: ${{ matrix.context }}
          file: ${{ matrix.context }}/${{ matrix.name }}.yaml
          push: true
          tags: |
            ${{ env.IMAGE_PREFIX }}/${{ matrix.name }}:v${{ env.VERSION }}
            ${{ env.IMAGE_PREFIX }}/${{ matrix.name }}:latest

      - name: Build and push (template image)
        if: ${{ startsWith(matrix.context, 'template-') }}
        uses: docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a # v7.3.0
        with:
          context: ${{ matrix.context }}
          push: true
          tags: |
            ${{ env.IMAGE_PREFIX }}/${{ matrix.name }}:v${{ env.VERSION }}
            ${{ env.IMAGE_PREFIX }}/${{ matrix.name }}:latest
```
