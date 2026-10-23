#include "xzuff3d/renderer.hpp"

#include <algorithm>
#include <cmath>
#include <fstream>
#include <limits>
#include <stdexcept>
#include <utility>

namespace xzuff3d {
namespace {
constexpr float kEpsilon = 1.0e-6F;
float dot(Vec3 a, Vec3 b) { return a.x*b.x+a.y*b.y+a.z*b.z; }
Vec3 cross(Vec3 a, Vec3 b) { return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x}; }
float length(Vec3 v) { return std::sqrt(dot(v,v)); }
Vec3 normalized(Vec3 v) { const float l=length(v); return l>kEpsilon ? v/l : Vec3{0,0,0}; }
float clamp01(float v) { return std::clamp(v,0.0F,1.0F); }
struct ViewVertex { Vec3 p; Vec3 n; };
struct ScreenVertex { float x,y,z; Vec3 normal; };
float edge(const ScreenVertex& a,const ScreenVertex& b,float x,float y) {
    return (x-a.x)*(b.y-a.y)-(y-a.y)*(b.x-a.x);
}
Vec3 rotateEuler(Vec3 p, Vec3 r) {
    const float cx=std::cos(r.x), sx=std::sin(r.x);
    const float cy=std::cos(r.y), sy=std::sin(r.y);
    const float cz=std::cos(r.z), sz=std::sin(r.z);
    // X then Y then Z rotation.
    p={p.x, p.y*cx-p.z*sx, p.y*sx+p.z*cx};
    p={p.x*cy+p.z*sy, p.y, -p.x*sy+p.z*cy};
    return {p.x*cz-p.y*sz,p.x*sz+p.y*cz,p.z};
}
ViewVertex mix(ViewVertex a,ViewVertex b,float t) {
    return {a.p+(b.p-a.p)*t,a.n+(b.n-a.n)*t};
}
std::vector<ViewVertex> clipPlane(const std::vector<ViewVertex>& in,float planeZ,bool keepGreater) {
    std::vector<ViewVertex> out;
    if(in.empty()) return out;
    ViewVertex prev=in.back();
    bool prevIn=keepGreater ? prev.p.z>=planeZ : prev.p.z<=planeZ;
    for(const auto& cur:in) {
        const bool curIn=keepGreater ? cur.p.z>=planeZ : cur.p.z<=planeZ;
        if(curIn != prevIn) {
            const float denom=cur.p.z-prev.p.z;
            if(std::abs(denom)>kEpsilon) out.push_back(mix(prev,cur,(planeZ-prev.p.z)/denom));
        }
        if(curIn) out.push_back(cur);
        prev=cur; prevIn=curIn;
    }
    return out;
}
Color shadedColor(Color base,Color light,float factor) {
    return {
        static_cast<std::uint8_t>(std::clamp(base.r*(light.r/255.0F)*factor,0.0F,255.0F)),
        static_cast<std::uint8_t>(std::clamp(base.g*(light.g/255.0F)*factor,0.0F,255.0F)),
        static_cast<std::uint8_t>(std::clamp(base.b*(light.b/255.0F)*factor,0.0F,255.0F))
    };
}
}

Vec3 Transform::applyPoint(Vec3 p) const noexcept {
    p={p.x*scale.x,p.y*scale.y,p.z*scale.z};
    return rotateEuler(p,rotation)+position;
}
Vec3 Transform::applyDirection(Vec3 n) const noexcept {
    // Inverse-scale normals before rotation to handle non-uniform scaling.
    n={scale.x!=0?n.x/scale.x:0,scale.y!=0?n.y/scale.y:0,scale.z!=0?n.z/scale.z:0};
    return normalized(rotateEuler(n,rotation));
}
Mesh Mesh::cube(float size) {
    if(!(size>0.0F)||!std::isfinite(size)) throw std::invalid_argument("cube size must be finite and positive");
    const float h=size*0.5F;
    const Vec3 p[]={{-h,-h,-h},{h,-h,-h},{h,h,-h},{-h,h,-h},
                    {-h,-h,h},{h,-h,h},{h,h,h},{-h,h,h}};
    const Triangle t[]={{0,2,1},{0,3,2},{4,5,6},{4,6,7},
        {0,1,5},{0,5,4},{3,7,6},{3,6,2},{0,4,7},{0,7,3},{1,2,6},{1,6,5}};
    Mesh m;
    m.vertices.reserve(8);
    for(auto v:p) m.vertices.push_back({v,normalized(v)});
    m.triangles.assign(std::begin(t),std::end(t));
    return m;
}

Framebuffer::Framebuffer(int w,int h) {
    if(w<=0||h<=0||w>8192||h>8192) throw std::invalid_argument("invalid framebuffer dimensions");
    width_=w; height_=h;
    pixels_=new std::uint8_t[static_cast<std::size_t>(w)*static_cast<std::size_t>(h)*3U]{};
}
Framebuffer::~Framebuffer(){ delete[] pixels_; }
Framebuffer::Framebuffer(Framebuffer&& o) noexcept : width_(o.width_),height_(o.height_),pixels_(o.pixels_) {
    o.width_=o.height_=0; o.pixels_=nullptr;
}
Framebuffer& Framebuffer::operator=(Framebuffer&& o) noexcept {
    if(this!=&o){ delete[] pixels_; width_=o.width_;height_=o.height_;pixels_=o.pixels_;o.width_=o.height_=0;o.pixels_=nullptr; }
    return *this;
}
void Framebuffer::clear(Color c) {
    const auto count=static_cast<std::size_t>(width_)*static_cast<std::size_t>(height_);
    for(std::size_t i=0;i<count;++i){pixels_[i*3]=c.r;pixels_[i*3+1]=c.g;pixels_[i*3+2]=c.b;}
}
void Framebuffer::writePpm(const std::string& path) const {
    std::ofstream out(path,std::ios::binary);
    if(!out) throw std::runtime_error("cannot open output image");
    out<<"P6\n"<<width_<<" "<<height_<<"\n255\n";
    out.write(reinterpret_cast<const char*>(pixels_),static_cast<std::streamsize>(static_cast<std::size_t>(width_)*height_*3U));
    if(!out) throw std::runtime_error("failed writing output image");
}
Renderer::Renderer(Framebuffer& target):target_(target),depth_(static_cast<std::size_t>(target.width())*target.height()) {}
void Renderer::clear(Color c){target_.clear(c);std::fill(depth_.begin(),depth_.end(),std::numeric_limits<float>::infinity());}
void Renderer::setAmbientLight(float i){if(!std::isfinite(i)||i<0)throw std::invalid_argument("ambient intensity must be finite and non-negative");ambientLight_=i;}
void Renderer::setDirectionalLight(const DirectionalLight& l){if(!std::isfinite(l.intensity)||l.intensity<0)throw std::invalid_argument("invalid directional light intensity");directional_=l;}
void Renderer::clearPointLights(){points_.clear();}
void Renderer::addPointLight(const PointLight& l){if(!std::isfinite(l.intensity)||l.intensity<0||!std::isfinite(l.range)||l.range<=0)throw std::invalid_argument("invalid point light");points_.push_back(l);}

void Renderer::renderMesh(const Mesh& mesh,const Transform& transform,const Camera& camera,const Material& material) {
    if(mesh.vertices.empty()||mesh.triangles.empty()) return;
    if(!(camera.verticalFovRadians>0&&camera.verticalFovRadians<3.13F)||
       !(camera.nearPlane>0)||!(camera.farPlane>camera.nearPlane))
        throw std::invalid_argument("invalid camera projection settings");
    const Vec3 forward=normalized(camera.target-camera.position);
    const Vec3 right=normalized(cross(forward,camera.up));
    const Vec3 up=normalized(cross(right,forward));
    if(length(forward)<kEpsilon||length(right)<kEpsilon) throw std::invalid_argument("invalid camera orientation");
    const float focal=(target_.height()*0.5F)/std::tan(camera.verticalFovRadians*0.5F);
    std::vector<ViewVertex> view;
    view.reserve(mesh.vertices.size());
    for(const auto& v:mesh.vertices) {
        const Vec3 world=transform.applyPoint(v.position);
        const Vec3 delta=world-camera.position;
        view.push_back({{dot(delta,right),dot(delta,up),dot(delta,forward)},transform.applyDirection(v.normal)});
    }
    for(const auto& tri:mesh.triangles) {
        if(tri.a>=view.size()||tri.b>=view.size()||tri.c>=view.size()) throw std::out_of_range("mesh triangle index out of range");
        std::vector<ViewVertex> polygon={view[tri.a],view[tri.b],view[tri.c]};
        polygon=clipPlane(polygon,camera.nearPlane,true);
        polygon=clipPlane(polygon,camera.farPlane,false);
        if(polygon.size()<3) continue;
        for(std::size_t fan=1;fan+1<polygon.size();++fan) {
            const ViewVertex vv[3]={polygon[0],polygon[fan],polygon[fan+1]};
            ScreenVertex s[3];
            for(int i=0;i<3;++i) {
                const float invz=1.0F/vv[i].p.z;
                s[i]={target_.width()*0.5F+vv[i].p.x*focal*invz,
                      target_.height()*0.5F-vv[i].p.y*focal*invz,vv[i].p.z,vv[i].n};
            }
            const float area=edge(s[0],s[1],s[2].x,s[2].y);
            if(std::abs(area)<kEpsilon) continue;
            const int minX=std::max(0,static_cast<int>(std::floor(std::min({s[0].x,s[1].x,s[2].x}))));
            const int maxX=std::min(target_.width()-1,static_cast<int>(std::ceil(std::max({s[0].x,s[1].x,s[2].x}))));
            const int minY=std::max(0,static_cast<int>(std::floor(std::min({s[0].y,s[1].y,s[2].y}))));
            const int maxY=std::min(target_.height()-1,static_cast<int>(std::ceil(std::max({s[0].y,s[1].y,s[2].y}))));
            for(int y=minY;y<=maxY;++y) for(int x=minX;x<=maxX;++x) {
                const float px=x+0.5F,py=y+0.5F;
                const float wa=edge(s[1],s[2],px,py)/area,wb=edge(s[2],s[0],px,py)/area,wc=edge(s[0],s[1],px,py)/area;
                if(wa<0||wb<0||wc<0) continue;
                const float z=wa*s[0].z+wb*s[1].z+wc*s[2].z;
                const auto idx=static_cast<std::size_t>(y)*target_.width()+x;
                if(z>=depth_[idx]) continue;
                Vec3 normal=normalized(s[0].normal*wa+s[1].normal*wb+s[2].normal*wc);
                if(length(normal)<kEpsilon) normal=normalized(cross(vv[1].p-vv[0].p,vv[2].p-vv[0].p));
                float illumination=ambientLight_*std::max(0.0F,material.ambient);
                const Vec3 toDirectional=normalized(directional_.direction*-1.0F);
                illumination+=std::max(0.0F,dot(normal,toDirectional))*directional_.intensity*std::max(0.0F,material.diffuse);
                Color light{255,255,255};
                // Directional-light tint is applied as a simple channel mix.
                light=shadedColor(light,directional_.color,1.0F);
                const Vec3 approximateWorld=camera.position+forward*z+right*((px-target_.width()*0.5F)*z/focal)+up*((target_.height()*0.5F-py)*z/focal);
                for(const auto& point:points_) {
                    const Vec3 delta=point.position-approximateWorld;
                    const float d=length(delta);
                    if(d<point.range&&d>kEpsilon) {
                        const float attenuation=1.0F-d/point.range;
                        const float amount=std::max(0.0F,dot(normal,delta/d))*point.intensity*attenuation*attenuation*std::max(0.0F,material.diffuse);
                        illumination+=amount;
                    }
                }
                illumination=std::clamp(illumination,0.0F,1.5F);
                const float lr=std::clamp((light.r/255.0F)*illumination,0.0F,1.0F);
                const float lg=std::clamp((light.g/255.0F)*illumination,0.0F,1.0F);
                const float lb=std::clamp((light.b/255.0F)*illumination,0.0F,1.0F);
                target_.pixels_[idx*3]=static_cast<std::uint8_t>(std::clamp(material.baseColor.r*lr,0.0F,255.0F));
                target_.pixels_[idx*3+1]=static_cast<std::uint8_t>(std::clamp(material.baseColor.g*lg,0.0F,255.0F));
                target_.pixels_[idx*3+2]=static_cast<std::uint8_t>(std::clamp(material.baseColor.b*lb,0.0F,255.0F));
                depth_[idx]=z;
            }
        }
    }
}
void Renderer::drawCube(Vec3 center,float size,float angle,Color color) {
    Transform t; t.position=center; t.rotation.y=angle;
    Material m; m.baseColor=color;
    Camera c; c.position={0,0,-4}; c.target={0,0,0};
    renderMesh(Mesh::cube(size),t,c,m);
}
} // namespace xzuff3d
