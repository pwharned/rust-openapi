

CREATE TABLE TEST(id INT PRIMARY KEY, id2 INT);



CREATE TABLE public.asset_types (
	type_id text NOT NULL,
	type_name text NOT NULL,
	CONSTRAINT asset_types_pkey PRIMARY KEY (type_id),
	CONSTRAINT asset_types_type_name_key UNIQUE (type_name)
);


CREATE TABLE public.brands (
	brand_id text NOT NULL,
	brand_name text NOT NULL,
	CONSTRAINT brand_pkey PRIMARY KEY (brand_id),
	CONSTRAINT brands_brand_name_key UNIQUE (brand_name)
);



CREATE TABLE public.collections (
	collection_id uuid DEFAULT gen_random_uuid() NOT NULL,
	collection_name text NOT NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	collection_description text NOT NULL,
	collection_owner text NOT NULL,
	collection_collaborators text NULL,
	CONSTRAINT asset_collections_collection_name_key UNIQUE (collection_name),
	CONSTRAINT asset_collections_pkey PRIMARY KEY (collection_id)
);



CREATE TABLE public.offering_types (
	offering_type_id text NOT NULL,
	offering_type_name text NOT NULL,
	CONSTRAINT offering_types_pk PRIMARY KEY (offering_type_id),
	CONSTRAINT offering_types_un UNIQUE (offering_type_id, offering_type_name)
);


CREATE TABLE public.practices (
	practice_id text NOT NULL,
	practice_name text NOT NULL,
	owning_brand text NOT NULL,
	CONSTRAINT practices_pkey PRIMARY KEY (practice_id),
	CONSTRAINT practices_practice_name_key UNIQUE (practice_name),
	CONSTRAINT practices_owning_brand_fkey FOREIGN KEY (owning_brand) REFERENCES public.brands(brand_id) ON DELETE CASCADE ON UPDATE CASCADE
);


CREATE TABLE public.products (
	product_id text NOT NULL,
	product_name text NOT NULL,
	owning_practice text NOT NULL,
	CONSTRAINT products_pkey PRIMARY KEY (product_id),
	CONSTRAINT products_owning_practice_fkey FOREIGN KEY (owning_practice) REFERENCES public.practices(practice_id) ON DELETE CASCADE ON UPDATE CASCADE
);



CREATE TABLE public.assets (
	asset_id uuid DEFAULT gen_random_uuid() NOT NULL,
	asset_name text NOT NULL,
	asset_owner text NOT NULL,
	asset_description text NOT NULL,
	asset_type text NOT NULL,
	asset_link text NOT NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NULL,
	"memberOfCollections" _text NULL,
	asset_offering_type text NULL,
	asset_brand text NOT NULL,
	asset_practice text NOT NULL,
	is_ip_cleared bool DEFAULT false NULL,
	is_sellable bool DEFAULT false NULL,
	asset_collaborators _text NULL,
	CONSTRAINT assets_asset_name_key UNIQUE (asset_name),
	CONSTRAINT assets_pkey PRIMARY KEY (asset_id),
	CONSTRAINT assets_asset_type_fkey FOREIGN KEY (asset_type) REFERENCES public.asset_types(type_id),
	CONSTRAINT assets_brand_fk FOREIGN KEY (asset_brand) REFERENCES public.brands(brand_id),
	CONSTRAINT assets_offering_type_fk FOREIGN KEY (asset_offering_type) REFERENCES public.offering_types(offering_type_id),
	CONSTRAINT assets_practice_fk FOREIGN KEY (asset_practice) REFERENCES public.practices(practice_id)
);



CREATE TABLE public.asset_collection (
	asset_id uuid NOT NULL,
	collection_id uuid NOT NULL,
	CONSTRAINT asset_collection_pkey PRIMARY KEY (asset_id, collection_id),
	CONSTRAINT asset_collection_asset_fk FOREIGN KEY (asset_id) REFERENCES public.assets(asset_id) ON DELETE CASCADE,
	CONSTRAINT asset_collection_collection_fk FOREIGN KEY (collection_id) REFERENCES public.collections(collection_id) ON DELETE CASCADE
);


CREATE TABLE public.asset_product (
	asset_id uuid NOT NULL,
	product_id text NOT NULL,
	CONSTRAINT asset_product_pk PRIMARY KEY (asset_id, product_id),
	CONSTRAINT asset_product_un UNIQUE (asset_id, product_id),
	CONSTRAINT asset_product_asset_fk FOREIGN KEY (asset_id) REFERENCES public.assets(asset_id) ON DELETE CASCADE,
	CONSTRAINT asset_product_product_fk FOREIGN KEY (product_id) REFERENCES public.products(product_id) ON DELETE CASCADE
);

CREATE TABLE public.asset_ratings (
	rating_id uuid DEFAULT gen_random_uuid() NOT NULL,
	rating_value float8 NOT NULL,
	createdby text NOT NULL,
	related_asset uuid NOT NULL,
	CONSTRAINT asset_ratings_pk PRIMARY KEY (rating_id),
	CONSTRAINT asset_ratings_assets_fk FOREIGN KEY (related_asset) REFERENCES public.assets(asset_id) ON DELETE CASCADE,
	CONSTRAINT asset_ratings_fk FOREIGN KEY (related_asset) REFERENCES public.assets(asset_id) ON DELETE CASCADE ON UPDATE CASCADE
);

